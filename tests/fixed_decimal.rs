use gpui_wasm::{Frontplane, Limits};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");
const FIXED_SCALE: i64 = 1_000_000;

const VIEWPORT_WIDTH: usize = 8;
const VIEWPORT_HEIGHT: usize = 16;
const SHIP_X: usize = 24;
const SHIP_Y: usize = 32;
const SHIP_VX: usize = 40;
const SHIP_VY: usize = 48;

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
