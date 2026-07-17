use std::collections::HashSet;

use gpui_wasm::{AudioEvent, DrawCommand, Event, Frontplane, HostEffect, Key, Limits, MenuItem};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");

const FIXED_SCALE: i64 = 1_000_000;
const SCORE: usize = 72;
const LIVES: usize = 76;
const LEVEL: usize = 80;
const FLAGS: usize = 84;
const INVULNERABILITY: usize = 92;
const LIFECYCLE: usize = 100;
const LIFECYCLE_TICKS: usize = 104;
const DEATH_ROTATION: usize = 120;
const SHIP_X: usize = 24;
const SHIP_Y: usize = 32;
const SHIP_VX: usize = 40;
const SHIP_VY: usize = 48;
const SHIP_DX: usize = 56;
const SHIP_DY: usize = 64;

const BULLET_BASE: usize = 256;
const BULLET_STRIDE: usize = 48;
const BULLET_CAPACITY: usize = 64;
const BULLET_ACTIVE: usize = 0;
const BULLET_X: usize = 8;
const BULLET_Y: usize = 16;
const BULLET_VX: usize = 24;
const BULLET_VY: usize = 32;
const BULLET_DISTANCE: usize = 40;

const ASTEROID_BASE: usize = 3328;
const ASTEROID_STRIDE: usize = 80;
const ASTEROID_CAPACITY: usize = 32;
const ASTEROID_ACTIVE: usize = 0;
const ASTEROID_SHAPE: usize = 8;
const ASTEROID_X: usize = 16;
const ASTEROID_Y: usize = 24;
const ASTEROID_VX: usize = 32;
const ASTEROID_VY: usize = 40;
const ASTEROID_RADIUS: usize = 48;
const ASTEROID_SPIN: usize = 72;
const ASTEROID_POINTS: usize = 76;

const PARTICLE_BASE: usize = 5888;
const PARTICLE_STRIDE: usize = 48;
const PARTICLE_CAPACITY: usize = 150;

const DEBRIS_BASE: usize = 13088;
const DEBRIS_STRIDE: usize = 80;
const DEBRIS_CAPACITY: usize = 4;

fn demo() -> Frontplane {
    demo_with_seed(0x5eed)
}

fn demo_with_seed(seed: u64) -> Frontplane {
    Frontplane::from_wat(ASTEROIDS_WAT, Limits::default())
        .and_then(|mut frontplane| {
            frontplane.configure()?;
            frontplane.init(seed, 1024.0, 768.0)?;
            Ok(frontplane)
        })
        .expect("the bundled WAT plugin should load")
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i64(bytes: &mut [u8], offset: usize, value: i64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn active_count(bytes: &[u8], base: usize, stride: usize, capacity: usize) -> usize {
    (0..capacity)
        .filter(|index| read_i32(bytes, base + index * stride) != 0)
        .count()
}

fn asteroid_body_count(commands: &[DrawCommand]) -> usize {
    commands
        .iter()
        .filter(|command| {
            matches!(
                command,
                DrawCommand::Path { id, .. } if (200..232).contains(id)
            )
        })
        .count()
}

fn text_exists(commands: &[DrawCommand], expected: &str) -> bool {
    commands
        .iter()
        .any(|command| matches!(command, DrawCommand::Text { text, .. } if text == expected))
}

fn tick_in_fuel_safe_slices(frontplane: &mut Frontplane, mut ticks: u32) {
    while ticks > 0 {
        let slice = ticks.min(5);
        frontplane.tick(slice).unwrap();
        frontplane.drain_audio();
        ticks -= slice;
    }
}

#[test]
fn seed_changes_the_deterministic_asteroid_field() {
    let mut first = demo_with_seed(1);
    let mut second = demo_with_seed(2);

    assert_ne!(first.render().unwrap(), second.render().unwrap());
}

#[test]
fn initial_asteroid_radius_distribution_reaches_the_source_twenty_to_fifty_range() {
    let records = (1..=12)
        .flat_map(|seed| {
            let mut frontplane = demo_with_seed(seed);
            let state = frontplane.snapshot().unwrap();
            (0..5)
                .map(|index| {
                    let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
                    (
                        read_i64(&state.bytes, address + ASTEROID_RADIUS),
                        read_i32(&state.bytes, address + ASTEROID_SPIN),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let radii = records.iter().map(|record| record.0).collect::<Vec<_>>();
    let spins = records.iter().map(|record| record.1).collect::<Vec<_>>();

    assert!(
        radii
            .iter()
            .all(|radius| (20_000_000..50_000_000).contains(radius))
    );
    assert!(radii.iter().any(|radius| *radius < 24_000_000));
    assert!(radii.iter().any(|radius| *radius > 48_000_000));
    assert!(spins.iter().all(|spin| (-16_666..=16_666).contains(spin)));
    assert!(spins.iter().any(|spin| spin.abs() > 1_000));
}

#[test]
fn plugin_declares_metadata_and_standard_menus() {
    let frontplane = demo();
    let metadata = frontplane.metadata();

    assert_eq!(metadata.title, "Vibesteroids — WAT");
    assert_eq!(
        metadata.menu_items,
        vec![
            MenuItem::action(1, "New Game", Some("Ctrl+N")),
            MenuItem::separator(),
            MenuItem::action(7, "Help / Controls", Some("F1")),
            MenuItem::action(6, "Quit", Some("Ctrl+Q")),
        ]
    );
    assert_eq!(
        (1..=6)
            .map(|id| {
                metadata
                    .synth_voices
                    .iter()
                    .filter(|voice| voice.program_id == id)
                    .count()
            })
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 1, 3, 5]
    );
}

#[test]
fn help_is_plugin_owned_and_freezes_simulation_until_closed() {
    let mut frontplane = demo();
    frontplane.event(Event::MenuAction(7)).unwrap();
    let before = frontplane.snapshot().unwrap();

    assert!(text_exists(
        &frontplane.render().unwrap().commands,
        "CONTROLS"
    ));
    frontplane.tick(10).unwrap();
    let mut after = frontplane.snapshot().unwrap();
    after.bytes[0..4].copy_from_slice(&before.bytes[0..4]);
    assert_eq!(after, before);

    frontplane.event(Event::MenuAction(7)).unwrap();
    assert!(!text_exists(
        &frontplane.render().unwrap().commands,
        "CONTROLS"
    ));
}

#[test]
fn auto_fire_is_a_latched_toggle_not_a_held_key() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::AutoFire)).unwrap();
    frontplane.tick(1).unwrap();
    frontplane.event(Event::KeyUp(Key::AutoFire)).unwrap();
    frontplane.tick(24).unwrap();
    let firing = frontplane.snapshot().unwrap();
    assert_eq!(
        active_count(&firing.bytes, BULLET_BASE, BULLET_STRIDE, BULLET_CAPACITY),
        2
    );

    frontplane.event(Event::KeyDown(Key::AutoFire)).unwrap();
    frontplane.tick(10).unwrap();
    let stopped = frontplane.snapshot().unwrap();
    assert_eq!(
        active_count(&stopped.bytes, BULLET_BASE, BULLET_STRIDE, BULLET_CAPACITY),
        2
    );
}

#[test]
fn kid_mode_preserves_lives_and_hides_the_ordinary_hud_during_a_collision() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let ship_x = read_i64(&state.bytes, SHIP_X);
    let ship_y = read_i64(&state.bytes, SHIP_Y);
    write_i32(&mut state.bytes, INVULNERABILITY, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_X, ship_x);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, ship_y);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20 * FIXED_SCALE,
    );
    frontplane.restore(&state).unwrap();
    frontplane.event(Event::KeyDown(Key::KidMode)).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();
    let frame = frontplane.render().unwrap();
    assert_eq!(read_i32(&after.bytes, LIVES), 3);
    assert_eq!(read_i32(&after.bytes, SCORE), 0);
    assert_eq!(read_i32(&after.bytes, LIFECYCLE), 1);
    assert!(!text_exists(&frame.commands, "SCORE"));
    assert!(!text_exists(&frame.commands, "LIVES"));
}

#[test]
fn death_blossom_consumes_one_use_rotates_and_fires_until_exact_tick_boundary() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::DeathBlossom)).unwrap();
    let activated = frontplane.snapshot().unwrap();
    assert_ne!(read_i32(&activated.bytes, FLAGS) & 128, 0);
    assert_eq!(read_i32(&activated.bytes, FLAGS) & 256, 0);
    assert!(text_exists(
        &frontplane.render().unwrap().commands,
        "DEATH BLOSSOM!"
    ));
    assert!(frontplane.drain_audio().iter().any(|event| event.id == 5));

    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(10).unwrap();
    let firing = frontplane.snapshot().unwrap();
    assert_eq!(read_i64(&firing.bytes, SHIP_VX), 0);
    assert_eq!(read_i64(&firing.bytes, SHIP_VY), 0);
    assert_eq!(read_i64(&firing.bytes, DEATH_ROTATION), 1_200_000);
    assert!(active_count(&firing.bytes, BULLET_BASE, BULLET_STRIDE, BULLET_CAPACITY) >= 5);

    tick_in_fuel_safe_slices(&mut frontplane, 618);
    let one_tick_before = frontplane.snapshot().unwrap();
    assert_ne!(read_i32(&one_tick_before.bytes, FLAGS) & 128, 0);
    frontplane.tick(1).unwrap();
    let completed = frontplane.snapshot().unwrap();
    assert_eq!(read_i32(&completed.bytes, FLAGS) & 128, 0);
    assert_eq!(read_i32(&completed.bytes, FLAGS) & 256, 0);

    frontplane.event(Event::KeyDown(Key::DeathBlossom)).unwrap();
    assert_eq!(frontplane.snapshot().unwrap(), completed);
}

#[test]
fn initial_state_has_a_real_hud_and_five_jagged_asteroids() {
    let mut frontplane = demo();
    let snapshot = frontplane.snapshot().unwrap();
    let frame = frontplane.render().expect("render should complete");

    assert_eq!(snapshot.schema, 3);
    assert_eq!(snapshot.bytes.len(), 16_384);
    assert_eq!(read_i32(&snapshot.bytes, SCORE), 0);
    assert_eq!(read_i32(&snapshot.bytes, LIVES), 3);
    assert_eq!(read_i32(&snapshot.bytes, LEVEL), 1);
    assert_eq!(
        active_count(
            &snapshot.bytes,
            ASTEROID_BASE,
            ASTEROID_STRIDE,
            ASTEROID_CAPACITY,
        ),
        5
    );

    assert_eq!(frame.background, 0x080b12ff);
    for text in ["VIBESTEROIDS", "SCORE", "000000", "LEVEL", "01", "LIVES"] {
        assert!(
            text_exists(&frame.commands, text),
            "missing HUD text {text}"
        );
    }
    assert_eq!(asteroid_body_count(&frame.commands), 5);
    assert!(
        !frame.commands.iter().any(
            |command| matches!(command, DrawCommand::Circle { id, .. } if (200..232).contains(id))
        ),
        "asteroids must be jagged vector paths, not round placeholders"
    );
    assert_eq!(
        frame
            .commands
            .iter()
            .filter(
                |command| matches!(command, DrawCommand::Path { id, .. } if (50..52).contains(id))
            )
            .count(),
        2,
        "three lives should render as two reserve-ship icons"
    );
}

#[test]
fn asteroid_paths_honor_each_seeded_eight_to_twelve_vertex_count() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    write_i32(&mut state.bytes, ASTEROID_BASE + ASTEROID_POINTS, 8);
    write_i32(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_STRIDE + ASTEROID_POINTS,
        12,
    );
    frontplane.restore(&state).unwrap();
    let frame = frontplane.render().unwrap();

    for (id, points) in [(200, 8), (201, 12)] {
        let segments = frame
            .commands
            .iter()
            .find_map(|command| match command {
                DrawCommand::Path {
                    id: actual,
                    segments,
                    ..
                } if *actual == id => Some(segments),
                _ => None,
            })
            .expect("asteroid path exists");
        assert_eq!(segments.len(), points + 1, "move/lines plus close for {id}");
    }
}

#[test]
fn title_splash_uses_the_source_240_tick_lifecycle() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    write_i32(&mut state.bytes, INVULNERABILITY, 10_000);
    frontplane.restore(&state).unwrap();
    assert!(text_exists(
        &frontplane.render().unwrap().commands,
        "VIBESTEROIDS"
    ));

    tick_in_fuel_safe_slices(&mut frontplane, 239);
    assert!(text_exists(
        &frontplane.render().unwrap().commands,
        "VIBESTEROIDS"
    ));
    frontplane.tick(1).unwrap();
    assert!(!text_exists(
        &frontplane.render().unwrap().commands,
        "VIBESTEROIDS"
    ));
}

#[test]
fn wat_owns_input_simulation_and_deterministic_state() {
    let mut first = demo();
    let mut second = demo();

    for frontplane in [&mut first, &mut second] {
        frontplane.event(Event::KeyDown(Key::Right)).unwrap();
        frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
        frontplane.tick(30).unwrap();
        frontplane.event(Event::KeyUp(Key::Thrust)).unwrap();
        frontplane.event(Event::KeyUp(Key::Right)).unwrap();
    }

    assert_eq!(first.snapshot().unwrap(), second.snapshot().unwrap());
    assert_eq!(first.render().unwrap(), second.render().unwrap());
}

#[test]
fn ship_uses_the_gentler_drag_coefficient() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    write_i64(&mut state.bytes, SHIP_VX, 2_500_000);
    write_i64(&mut state.bytes, SHIP_VY, -1_250_000);
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(read_i64(&after.bytes, SHIP_VX), 2_487_500);
    assert_eq!(read_i64(&after.bytes, SHIP_VY), -1_243_750);
}

#[test]
fn level_one_and_twenty_use_the_source_difficulty_endpoints() {
    let cases = [
        (1, -82_916, 83_237, -996_530, -5_625_000, 16_u32),
        (20, -138_193, 165_895, -986_144, -12_750_000, 8_u32),
    ];

    for (level, thrust_vy, rotated_dx, rotated_dy, bullet_vy, fire_ticks) in cases {
        let mut thrusting = demo();
        let mut state = thrusting.snapshot().unwrap();
        write_i32(&mut state.bytes, LEVEL, level);
        thrusting.restore(&state).unwrap();
        thrusting.event(Event::KeyDown(Key::Thrust)).unwrap();
        thrusting.tick(1).unwrap();
        assert_eq!(
            read_i64(&thrusting.snapshot().unwrap().bytes, SHIP_VY),
            thrust_vy,
            "level {level} acceleration"
        );

        let mut rotating = demo();
        let mut state = rotating.snapshot().unwrap();
        write_i32(&mut state.bytes, LEVEL, level);
        rotating.restore(&state).unwrap();
        rotating.event(Event::KeyDown(Key::Right)).unwrap();
        rotating.tick(1).unwrap();
        let rotated = rotating.snapshot().unwrap();
        assert_eq!(read_i64(&rotated.bytes, SHIP_DX), rotated_dx);
        assert_eq!(read_i64(&rotated.bytes, SHIP_DY), rotated_dy);

        let mut firing = demo();
        let mut state = firing.snapshot().unwrap();
        write_i32(&mut state.bytes, LEVEL, level);
        firing.restore(&state).unwrap();
        firing.event(Event::KeyDown(Key::Fire)).unwrap();
        firing.tick(1).unwrap();
        assert_eq!(
            read_i64(&firing.snapshot().unwrap().bytes, BULLET_BASE + BULLET_VY),
            bullet_vy,
            "level {level} bullet speed"
        );
        firing.tick(fire_ticks - 1).unwrap();
        assert_eq!(
            active_count(
                &firing.snapshot().unwrap().bytes,
                BULLET_BASE,
                BULLET_STRIDE,
                BULLET_CAPACITY,
            ),
            1,
            "level {level} fired too early"
        );
        firing.tick(1).unwrap();
        assert_eq!(
            active_count(
                &firing.snapshot().unwrap().bytes,
                BULLET_BASE,
                BULLET_STRIDE,
                BULLET_CAPACITY,
            ),
            2,
            "level {level} did not fire at its exact cadence"
        );
    }
}

#[test]
fn asteroid_component_cap_applies_below_level_twenty_and_disappears_at_twenty() {
    for (level, should_cap) in [(1, true), (20, false)] {
        let mut frontplane = demo();
        let mut state = frontplane.snapshot().unwrap();
        let asteroid_x = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_X);
        let asteroid_y = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_Y);
        write_i32(&mut state.bytes, LEVEL, level);
        write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 10_000_000);
        write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 10_000_000);
        write_i64(
            &mut state.bytes,
            ASTEROID_BASE + ASTEROID_RADIUS,
            40_000_000,
        );
        write_i32(&mut state.bytes, BULLET_BASE + BULLET_ACTIVE, 1);
        write_i64(
            &mut state.bytes,
            BULLET_BASE + BULLET_X,
            asteroid_x + 10_000_000,
        );
        write_i64(
            &mut state.bytes,
            BULLET_BASE + BULLET_Y,
            asteroid_y + 10_000_000,
        );
        write_i64(&mut state.bytes, BULLET_BASE + BULLET_VX, 0);
        write_i64(&mut state.bytes, BULLET_BASE + BULLET_VY, 0);
        frontplane.restore(&state).unwrap();

        frontplane.tick(1).unwrap();
        let after = frontplane.snapshot().unwrap();
        let child_components = (0..ASTEROID_CAPACITY)
            .filter_map(|index| {
                let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
                (read_i32(&after.bytes, address + ASTEROID_ACTIVE) != 0
                    && read_i64(&after.bytes, address + ASTEROID_RADIUS) == 24_000_000)
                    .then(|| read_i64(&after.bytes, address + ASTEROID_VX).abs())
            })
            .collect::<Vec<_>>();
        assert_eq!(child_components.len(), 2);
        assert_eq!(
            child_components
                .iter()
                .all(|component| *component <= 1_000_000),
            should_cap,
            "level {level} component cap"
        );
    }
}

#[test]
fn firing_emits_semantic_audio_and_a_bullet_draw_command() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();

    let audio = frontplane.drain_audio();
    assert!(
        audio.contains(&AudioEvent::fire()),
        "recorded audio: {audio:?}"
    );
    assert!(frontplane.render().unwrap().commands.iter().any(|command| {
        matches!(
            command,
            DrawCommand::Circle {
                id: 100,
                filled: true,
                ..
            }
        )
    }));
}

#[test]
fn holding_fire_creates_multiple_simultaneous_bullets() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(45).unwrap();
    let snapshot = frontplane.snapshot().unwrap();

    assert!(active_count(&snapshot.bytes, BULLET_BASE, BULLET_STRIDE, BULLET_CAPACITY,) >= 3);
}

#[test]
fn a_new_bullet_records_exact_fixed_distance_traveled() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();
    let snapshot = frontplane.snapshot().unwrap();

    assert_eq!(
        read_i64(&snapshot.bytes, BULLET_BASE + BULLET_DISTANCE),
        5_625_000
    );
}

#[test]
fn bullet_expires_at_half_the_live_viewport_diagonal() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();
    frontplane.event(Event::KeyUp(Key::Fire)).unwrap();

    frontplane.tick(112).unwrap();
    let before_boundary = frontplane.snapshot().unwrap();
    assert_eq!(
        read_i64(&before_boundary.bytes, BULLET_BASE + BULLET_DISTANCE),
        635_625_000
    );
    assert_eq!(
        read_i32(&before_boundary.bytes, BULLET_BASE + BULLET_ACTIVE),
        1
    );

    frontplane.tick(1).unwrap();
    assert_eq!(
        read_i32(
            &frontplane.snapshot().unwrap().bytes,
            BULLET_BASE + BULLET_ACTIVE,
        ),
        0
    );
}

#[test]
fn a_large_asteroid_hit_splits_into_two_children_and_awards_80() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let asteroid_x = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_X);
    let asteroid_y = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_Y);

    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        40 * FIXED_SCALE,
    );
    write_i32(&mut state.bytes, BULLET_BASE + BULLET_ACTIVE, 1);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_X, asteroid_x);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_Y, asteroid_y);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_VX, 0);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_VY, 0);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_DISTANCE, 0);
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();
    let child_radii = (0..ASTEROID_CAPACITY)
        .filter_map(|index| {
            let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
            (read_i32(&after.bytes, address + ASTEROID_ACTIVE) != 0)
                .then(|| read_i64(&after.bytes, address + ASTEROID_RADIUS))
        })
        .filter(|radius| *radius == 24 * FIXED_SCALE)
        .count();
    let child_offsets = (0..ASTEROID_CAPACITY)
        .filter_map(|index| {
            let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
            (read_i32(&after.bytes, address + ASTEROID_ACTIVE) != 0
                && read_i64(&after.bytes, address + ASTEROID_RADIUS) == 24 * FIXED_SCALE)
                .then(|| {
                    (
                        read_i64(&after.bytes, address + ASTEROID_X) - asteroid_x,
                        read_i64(&after.bytes, address + ASTEROID_Y) - asteroid_y,
                    )
                })
        })
        .collect::<HashSet<_>>();

    assert_eq!(read_i32(&after.bytes, SCORE), 80);
    assert_eq!(
        active_count(
            &after.bytes,
            ASTEROID_BASE,
            ASTEROID_STRIDE,
            ASTEROID_CAPACITY,
        ),
        6
    );
    assert_eq!(child_radii, 2);
    assert_ne!(
        child_offsets,
        HashSet::from([(-6_000_000, 0), (6_000_000, 6_000_000)]),
        "children must use seeded ±20-pixel offsets, not a fixed template"
    );
    assert_eq!(
        active_count(
            &after.bytes,
            PARTICLE_BASE,
            PARTICLE_STRIDE,
            PARTICLE_CAPACITY,
        ),
        20
    );
    assert!(frontplane.drain_audio().iter().any(|event| event.id == 2));
    let frame = frontplane.render().unwrap();
    assert_eq!(asteroid_body_count(&frame.commands), 6);
    assert_eq!(
        frame
            .commands
            .iter()
            .filter(
                |command| matches!(command, DrawCommand::Circle { id, .. } if (500..650).contains(id))
            )
            .count(),
        20
    );
}

#[test]
fn a_terminal_asteroid_hit_destroys_it_and_awards_120() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let asteroid_x = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_X);
    let asteroid_y = read_i64(&state.bytes, ASTEROID_BASE + ASTEROID_Y);

    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20 * FIXED_SCALE,
    );
    write_i32(&mut state.bytes, BULLET_BASE + BULLET_ACTIVE, 1);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_X, asteroid_x);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_Y, asteroid_y);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_VX, 0);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_VY, 0);
    write_i64(&mut state.bytes, BULLET_BASE + BULLET_DISTANCE, 0);
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(read_i32(&after.bytes, SCORE), 120);
    assert_eq!(
        active_count(
            &after.bytes,
            ASTEROID_BASE,
            ASTEROID_STRIDE,
            ASTEROID_CAPACITY,
        ),
        4
    );
}

#[test]
fn clearing_a_wave_advances_level_and_spawns_one_more_parent() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    for index in 0..ASTEROID_CAPACITY {
        write_i32(
            &mut state.bytes,
            ASTEROID_BASE + index * ASTEROID_STRIDE + ASTEROID_ACTIVE,
            0,
        );
    }
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(read_i32(&after.bytes, LEVEL), 2);
    assert_eq!(
        active_count(
            &after.bytes,
            ASTEROID_BASE,
            ASTEROID_STRIDE,
            ASTEROID_CAPACITY,
        ),
        6
    );
}

#[test]
fn ship_collision_creates_debris_then_respawns_or_reaches_game_over() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let ship_x = read_i64(&state.bytes, SHIP_X);
    let ship_y = read_i64(&state.bytes, SHIP_Y);

    write_i32(&mut state.bytes, INVULNERABILITY, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_X, ship_x);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, ship_y);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20 * FIXED_SCALE,
    );
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let exploding = frontplane.snapshot().unwrap();
    assert_eq!(read_i32(&exploding.bytes, LIVES), 2);
    assert_eq!(read_i32(&exploding.bytes, LIFECYCLE), 1);
    assert_eq!(
        active_count(
            &exploding.bytes,
            DEBRIS_BASE,
            DEBRIS_STRIDE,
            DEBRIS_CAPACITY,
        ),
        4
    );
    let explosion_frame = frontplane.render().unwrap();
    assert!(
        !explosion_frame
            .commands
            .iter()
            .any(|command| matches!(command, DrawCommand::Path { id: 1, .. }))
    );
    assert_eq!(
        explosion_frame
            .commands
            .iter()
            .filter(
                |command| matches!(command, DrawCommand::Path { id, .. } if (700..704).contains(id))
            )
            .count(),
        4
    );

    frontplane.tick(120).unwrap();
    let respawned = frontplane.snapshot().unwrap();
    assert_eq!(read_i32(&respawned.bytes, LIFECYCLE), 0);
    assert_eq!(read_i32(&respawned.bytes, LIVES), 2);

    let mut final_life = demo();
    let mut state = final_life.snapshot().unwrap();
    let ship_x = read_i64(&state.bytes, SHIP_X);
    let ship_y = read_i64(&state.bytes, SHIP_Y);
    write_i32(&mut state.bytes, LIVES, 1);
    write_i32(&mut state.bytes, INVULNERABILITY, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_X, ship_x);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, ship_y);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20 * FIXED_SCALE,
    );
    final_life.restore(&state).unwrap();

    final_life.tick(121).unwrap();
    let game_over = final_life.snapshot().unwrap();
    assert_eq!(read_i32(&game_over.bytes, LIFECYCLE), 3);
    assert!(text_exists(
        &final_life.render().unwrap().commands,
        "GAME OVER"
    ));
}

#[test]
fn explosion_and_respawn_lifecycles_keep_world_effects_advancing() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let ship_x = read_i64(&state.bytes, SHIP_X);
    let ship_y = read_i64(&state.bytes, SHIP_Y);
    write_i32(&mut state.bytes, INVULNERABILITY, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_X, ship_x);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, ship_y);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20_000_000,
    );
    frontplane.restore(&state).unwrap();
    frontplane.tick(1).unwrap();
    let exploding = frontplane.snapshot().unwrap();
    let debris_x = read_i64(&exploding.bytes, DEBRIS_BASE + 8);
    let debris_vx = read_i64(&exploding.bytes, DEBRIS_BASE + 24);
    let particle_life = read_i32(&exploding.bytes, PARTICLE_BASE + 4);

    frontplane.tick(1).unwrap();
    let advancing = frontplane.snapshot().unwrap();
    assert_eq!(
        read_i64(&advancing.bytes, DEBRIS_BASE + 8),
        debris_x + debris_vx
    );
    assert_eq!(
        read_i64(&advancing.bytes, DEBRIS_BASE + 24),
        debris_vx * 990_000 / FIXED_SCALE
    );
    assert_eq!(
        read_i32(&advancing.bytes, PARTICLE_BASE + 4),
        particle_life - 1
    );
}

#[test]
fn respawn_radius_shrinks_at_300_ticks_and_bomb_fires_at_600() {
    let mut shrinking = demo();
    let mut state = shrinking.snapshot().unwrap();
    let center_x = read_i64(&state.bytes, SHIP_X);
    let center_y = read_i64(&state.bytes, SHIP_Y);
    write_i32(&mut state.bytes, LIFECYCLE, 2);
    write_i32(&mut state.bytes, LIFECYCLE_TICKS, 298);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_X,
        center_x + 100_000_000,
    );
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, center_y);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VX, 0);
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_VY, 0);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20_000_000,
    );
    shrinking.restore(&state).unwrap();

    shrinking.tick(1).unwrap();
    assert_eq!(
        read_i32(&shrinking.snapshot().unwrap().bytes, LIFECYCLE),
        2,
        "96-pixel zone must still block at tick 299"
    );
    shrinking.tick(1).unwrap();
    assert_eq!(
        read_i32(&shrinking.snapshot().unwrap().bytes, LIFECYCLE),
        0,
        "48-pixel zone must allow respawn at tick 300"
    );

    let mut bombing = demo();
    let mut state = bombing.snapshot().unwrap();
    let center_x = read_i64(&state.bytes, SHIP_X);
    let center_y = read_i64(&state.bytes, SHIP_Y);
    for index in 0..ASTEROID_CAPACITY {
        write_i32(
            &mut state.bytes,
            ASTEROID_BASE + index * ASTEROID_STRIDE + ASTEROID_ACTIVE,
            0,
        );
    }
    write_i32(&mut state.bytes, LIFECYCLE, 2);
    write_i32(&mut state.bytes, LIFECYCLE_TICKS, 598);
    write_i32(&mut state.bytes, ASTEROID_BASE + ASTEROID_ACTIVE, 1);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_X,
        center_x + 60_000_000,
    );
    write_i64(&mut state.bytes, ASTEROID_BASE + ASTEROID_Y, center_y);
    write_i64(
        &mut state.bytes,
        ASTEROID_BASE + ASTEROID_RADIUS,
        20_000_000,
    );
    let far = ASTEROID_BASE + ASTEROID_STRIDE;
    write_i32(&mut state.bytes, far + ASTEROID_ACTIVE, 1);
    write_i64(&mut state.bytes, far + ASTEROID_X, center_x + 300_000_000);
    write_i64(&mut state.bytes, far + ASTEROID_Y, center_y);
    write_i64(&mut state.bytes, far + ASTEROID_RADIUS, 20_000_000);
    bombing.restore(&state).unwrap();

    bombing.tick(1).unwrap();
    assert_eq!(
        read_i32(&bombing.snapshot().unwrap().bytes, ASTEROID_BASE),
        1
    );
    bombing.tick(1).unwrap();
    let after_bomb = bombing.snapshot().unwrap();
    assert_eq!(read_i32(&after_bomb.bytes, ASTEROID_BASE), 0);
    assert_eq!(read_i32(&after_bomb.bytes, far + ASTEROID_ACTIVE), 1);
    assert_eq!(read_i32(&after_bomb.bytes, SCORE), 0);
    assert_eq!(read_i32(&after_bomb.bytes, LIFECYCLE), 0);
}

#[test]
fn pause_freezes_everything_except_the_clock() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(10).unwrap();
    frontplane.event(Event::KeyDown(Key::Pause)).unwrap();
    let before = frontplane.snapshot().unwrap();

    frontplane.tick(30).unwrap();
    let mut after = frontplane.snapshot().unwrap();
    after.bytes[0..4].copy_from_slice(&before.bytes[0..4]);

    assert_eq!(after, before);
}

#[test]
fn rendering_is_pure_and_does_not_advance_gameplay_rng() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(1).unwrap();
    let before = frontplane.snapshot().unwrap();
    let first = frontplane.render().unwrap();
    let second = frontplane.render().unwrap();

    assert_eq!(first, second);
    assert_eq!(frontplane.snapshot().unwrap(), before);
}

#[test]
fn snapshots_restore_the_exact_rendered_game_state() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(12).unwrap();
    let snapshot = frontplane.snapshot().unwrap();
    let expected_frame = frontplane.render().unwrap();

    frontplane.tick(40).unwrap();
    assert_ne!(frontplane.render().unwrap(), expected_frame);
    frontplane.restore(&snapshot).unwrap();

    assert_eq!(frontplane.render().unwrap(), expected_frame);
}

#[test]
fn mismatched_snapshots_are_rejected_without_mutating_state() {
    let mut frontplane = demo();
    frontplane.tick(7).unwrap();
    let current = frontplane.snapshot().unwrap();
    let mut wrong_schema = current.clone();
    wrong_schema.schema += 1;
    let mut wrong_length = current.clone();
    wrong_length.bytes.pop();

    for invalid in [&wrong_schema, &wrong_length] {
        assert!(frontplane.restore(invalid).is_err());
        assert_eq!(frontplane.snapshot().unwrap(), current);
    }
}

#[test]
fn a_full_bullet_pool_fails_closed_without_corrupting_asteroids() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    for index in 0..BULLET_CAPACITY {
        write_i32(
            &mut state.bytes,
            BULLET_BASE + index * BULLET_STRIDE + BULLET_ACTIVE,
            1,
        );
        write_i64(
            &mut state.bytes,
            BULLET_BASE + index * BULLET_STRIDE + BULLET_DISTANCE,
            0,
        );
    }
    let asteroid_invariants = (0..ASTEROID_CAPACITY)
        .map(|index| {
            let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
            (
                read_i32(&state.bytes, address + ASTEROID_ACTIVE),
                read_i32(&state.bytes, address + ASTEROID_SHAPE),
                read_i64(&state.bytes, address + ASTEROID_RADIUS),
            )
        })
        .collect::<Vec<_>>();
    frontplane.restore(&state).unwrap();

    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(
        (0..ASTEROID_CAPACITY)
            .map(|index| {
                let address = ASTEROID_BASE + index * ASTEROID_STRIDE;
                (
                    read_i32(&after.bytes, address + ASTEROID_ACTIVE),
                    read_i32(&after.bytes, address + ASTEROID_SHAPE),
                    read_i64(&after.bytes, address + ASTEROID_RADIUS),
                )
            })
            .collect::<Vec<_>>(),
        asteroid_invariants
    );
}

#[test]
fn new_game_is_a_generic_menu_event_and_quit_remains_host_owned() {
    let mut frontplane = demo();
    let initial = frontplane.snapshot().unwrap();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(20).unwrap();
    assert_ne!(frontplane.snapshot().unwrap(), initial);
    let _redraws = frontplane.drain_effects();

    frontplane.event(Event::MenuAction(1)).unwrap();
    assert_eq!(frontplane.snapshot().unwrap(), initial);
    frontplane.event(Event::MenuAction(6)).unwrap();

    assert_eq!(frontplane.drain_effects(), vec![HostEffect::Quit]);
}

#[test]
fn losing_focus_releases_held_controls() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.event(Event::Focus(false)).unwrap();
    let before = frontplane.snapshot().unwrap();
    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_ne!(before, after, "the clock should advance");
    assert!(
        frontplane.drain_audio().is_empty(),
        "thrust must be released"
    );
}
