use gpui_wasm::{Event, Frontplane, Limits};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");
const FIXED_SCALE: i64 = 1_000_000;

const VIEWPORT_WIDTH: usize = 8;
const VIEWPORT_HEIGHT: usize = 16;
const SHIP_X: usize = 24;
const SHIP_Y: usize = 32;
const SHIP_VX: usize = 40;
const SHIP_VY: usize = 48;
const RNG: usize = 4;
const BULLET_BASE: usize = 256;
const ASTEROID_BASE: usize = 3328;
const STAR_BASE: usize = 13_408;

fn demo() -> Frontplane {
    Frontplane::from_wat(ASTEROIDS_WAT, Limits::default())
        .and_then(|mut frontplane| {
            frontplane.configure()?;
            frontplane.init(0x5eed, 1024.0, 768.0)?;
            Ok(frontplane)
        })
        .expect("the bundled WAT plugin should load")
}

fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn write_i64(bytes: &mut [u8], offset: usize, value: i64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// Removes the only region where IEEE-754 operations may bridge the WAT ABI,
/// leaving the application body available for a mechanically enforced audit.
fn without_float_adapter(source: &str) -> String {
    const BEGIN: &str = ";; FLOAT ADAPTER BEGIN";
    const END: &str = ";; FLOAT ADAPTER END";

    let begin = source
        .find(BEGIN)
        .expect("WAT must mark the single float adapter boundary");
    let end = source
        .find(END)
        .expect("WAT must close the single float adapter boundary");
    assert!(begin < end, "float adapter markers are reversed");

    // Both sides are inspected by the caller. Replacing the adapter with a
    // newline keeps tokens on opposite sides from being accidentally joined.
    format!("{}\n{}", &source[..begin], &source[end + END.len()..])
}

#[test]
fn simulation_state_is_schema_three_decimal_fixed_point() {
    let mut frontplane = demo();
    let snapshot = frontplane.snapshot().unwrap();

    assert_eq!(snapshot.schema, 3);
    assert_eq!(snapshot.bytes.len(), 16_384);
    assert_eq!(
        read_i64(&snapshot.bytes, VIEWPORT_WIDTH),
        1024 * FIXED_SCALE
    );
    assert_eq!(
        read_i64(&snapshot.bytes, VIEWPORT_HEIGHT),
        768 * FIXED_SCALE
    );
    assert_eq!(read_i64(&snapshot.bytes, SHIP_X), 512 * FIXED_SCALE);
    assert_eq!(read_i64(&snapshot.bytes, SHIP_Y), 384 * FIXED_SCALE);

    let mut moving = snapshot;
    write_i64(&mut moving.bytes, SHIP_VX, 2_500_000);
    write_i64(&mut moving.bytes, SHIP_VY, -1_250_000);
    frontplane.restore(&moving).unwrap();
    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(read_i64(&after.bytes, SHIP_VX), 2_487_500);
    assert_eq!(read_i64(&after.bytes, SHIP_VY), -1_243_750);
}

#[test]
fn float_operations_exist_only_in_the_marked_host_scalar_adapter() {
    let application = without_float_adapter(ASTEROIDS_WAT);
    let forbidden = application
        .split(|character: char| character.is_whitespace() || matches!(character, '(' | ')'))
        .filter(|token| token.starts_with("f32.") && *token != "f32.const")
        .collect::<Vec<_>>();

    assert!(
        forbidden.is_empty(),
        "simulation/render arithmetic escaped the float adapter: {forbidden:?}"
    );
    assert!(!application.contains("f32.load"));
    assert!(!application.contains("f32.store"));
}

#[test]
fn resize_translates_world_by_center_delta_and_regenerates_only_stars() {
    let mut frontplane = demo();
    let mut before = frontplane.snapshot().unwrap();
    let asteroid_x = read_i64(&before.bytes, ASTEROID_BASE + 16);
    let asteroid_y = read_i64(&before.bytes, ASTEROID_BASE + 24);
    let rng = i32::from_le_bytes(before.bytes[RNG..RNG + 4].try_into().unwrap());
    let stars = before.bytes[STAR_BASE..STAR_BASE + 1_600].to_vec();

    before.bytes[BULLET_BASE..BULLET_BASE + 4].copy_from_slice(&1_i32.to_le_bytes());
    write_i64(&mut before.bytes, BULLET_BASE + 8, 100 * FIXED_SCALE);
    write_i64(&mut before.bytes, BULLET_BASE + 16, 200 * FIXED_SCALE);
    write_i64(&mut before.bytes, BULLET_BASE + 24, 1_234_567);
    frontplane.restore(&before).unwrap();

    frontplane
        .event(Event::Viewport {
            width: 1600.0,
            height: 900.0,
        })
        .unwrap();
    let after = frontplane.snapshot().unwrap();
    let center_dx = 288 * FIXED_SCALE;
    let center_dy = 66 * FIXED_SCALE;

    assert_eq!(read_i64(&after.bytes, VIEWPORT_WIDTH), 1600 * FIXED_SCALE);
    assert_eq!(read_i64(&after.bytes, VIEWPORT_HEIGHT), 900 * FIXED_SCALE);
    assert_eq!(
        read_i64(&after.bytes, SHIP_X),
        512 * FIXED_SCALE + center_dx
    );
    assert_eq!(
        read_i64(&after.bytes, SHIP_Y),
        384 * FIXED_SCALE + center_dy
    );
    assert_eq!(
        read_i64(&after.bytes, BULLET_BASE + 8),
        100 * FIXED_SCALE + center_dx
    );
    assert_eq!(
        read_i64(&after.bytes, BULLET_BASE + 16),
        200 * FIXED_SCALE + center_dy
    );
    assert_eq!(read_i64(&after.bytes, BULLET_BASE + 24), 1_234_567);
    assert_eq!(
        read_i64(&after.bytes, ASTEROID_BASE + 16),
        asteroid_x + center_dx
    );
    assert_eq!(
        read_i64(&after.bytes, ASTEROID_BASE + 24),
        asteroid_y + center_dy
    );
    assert_eq!(
        i32::from_le_bytes(after.bytes[RNG..RNG + 4].try_into().unwrap()),
        rng,
        "star regeneration must not consume gameplay RNG"
    );
    assert_ne!(&after.bytes[STAR_BASE..STAR_BASE + 1_600], stars);
}

#[test]
fn audio_oscillators_envelopes_and_filters_are_decimal_fixed_point() {
    let source = include_str!("../src/main.rs");
    let fixed = source
        .split_once("// FIXED_AUDIO_BEGIN")
        .expect("fixed-audio implementation start marker")
        .1
        .split_once("// FIXED_AUDIO_END")
        .expect("fixed-audio implementation end marker")
        .0;

    assert!(!fixed.contains("f32"), "fixed audio contains f32: {fixed}");
    assert!(!fixed.contains("f64"), "fixed audio contains f64: {fixed}");
}
