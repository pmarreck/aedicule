use gpui_wasm::{AudioEvent, DrawCommand, Event, Frontplane, HostEffect, Key, Limits, MenuItem};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");

const FIXED_SCALE: i64 = 1_000_000;
const SCORE: usize = 72;
const LIVES: usize = 76;
const LEVEL: usize = 80;
const INVULNERABILITY: usize = 92;
const LIFECYCLE: usize = 100;
const SHIP_X: usize = 24;
const SHIP_Y: usize = 32;
const SHIP_VX: usize = 40;
const SHIP_VY: usize = 48;

const BULLET_BASE: usize = 256;
const BULLET_STRIDE: usize = 48;
const BULLET_CAPACITY: usize = 64;
const BULLET_ACTIVE: usize = 0;
const BULLET_X: usize = 8;
const BULLET_Y: usize = 16;
const BULLET_VX: usize = 24;
const BULLET_VY: usize = 32;
const BULLET_LIFE: usize = 40;

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

#[test]
fn seed_changes_the_deterministic_asteroid_field() {
    let mut first = demo_with_seed(1);
    let mut second = demo_with_seed(2);

    assert_ne!(first.render().unwrap(), second.render().unwrap());
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
            MenuItem::action(6, "Quit", Some("Ctrl+Q")),
        ]
    );
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
fn a_new_bullet_consumes_exactly_one_lifetime_tick() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();
    let snapshot = frontplane.snapshot().unwrap();

    assert_eq!(read_i32(&snapshot.bytes, BULLET_BASE + BULLET_LIFE), 89);
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
    write_i32(&mut state.bytes, BULLET_BASE + BULLET_LIFE, 90);
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
    write_i32(&mut state.bytes, BULLET_BASE + BULLET_LIFE, 90);
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
        write_i32(
            &mut state.bytes,
            BULLET_BASE + index * BULLET_STRIDE + BULLET_LIFE,
            90,
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
