use gpui_wasm::{AudioEvent, DrawCommand, Event, Frontplane, HostEffect, Key, Limits, MenuItem};

const ASTEROIDS_WAT: &str = include_str!("../plugins/vibesteroids.wat");

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

#[test]
fn seed_changes_the_deterministic_asteroid_field() {
    let mut first = demo_with_seed(1);
    let mut second = demo_with_seed(2);

    assert_ne!(first.render().unwrap(), second.render().unwrap());
}

#[test]
fn plugin_declares_localizable_metadata_and_standard_menus() {
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
fn render_returns_a_complete_bounded_vector_frame() {
    let mut frontplane = demo();
    let frame = frontplane.render().expect("render should complete");

    assert_eq!(frame.background, 0x080b12ff);
    assert!(frame.commands.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text == "VIBESTEROIDS"
    )));
    assert_eq!(
        frame
            .commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Circle { .. }))
            .count(),
        3
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
fn a_new_bullet_consumes_exactly_one_lifetime_tick() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Fire)).unwrap();
    frontplane.tick(1).unwrap();
    let snapshot = frontplane.snapshot().unwrap();
    let lifetime = i32::from_le_bytes(snapshot.bytes[80..84].try_into().unwrap());

    assert_eq!(lifetime, 89);
}

#[test]
fn collision_resolution_changes_score_and_emits_explosion_audio() {
    let mut frontplane = demo();
    let mut state = frontplane.snapshot().unwrap();
    let asteroid_x = state.bytes[96..100].to_vec();
    let asteroid_y = state.bytes[100..104].to_vec();
    state.bytes[60..64].copy_from_slice(&1_i32.to_le_bytes());
    state.bytes[64..68].copy_from_slice(&asteroid_x);
    state.bytes[68..72].copy_from_slice(&asteroid_y);
    state.bytes[72..76].copy_from_slice(&0_f32.to_le_bytes());
    state.bytes[76..80].copy_from_slice(&0_f32.to_le_bytes());
    state.bytes[80..84].copy_from_slice(&90_i32.to_le_bytes());
    frontplane.restore(&state).unwrap();

    frontplane.tick(1).unwrap();
    let after = frontplane.snapshot().unwrap();
    let score = i32::from_le_bytes(after.bytes[40..44].try_into().unwrap());
    let asteroid_active = i32::from_le_bytes(after.bytes[116..120].try_into().unwrap());

    assert_eq!((score, asteroid_active), (80, 0));
    assert!(frontplane.drain_audio().iter().any(|event| event.id == 2));
}

#[test]
fn pause_freezes_gameplay_positions() {
    let mut frontplane = demo();
    frontplane.event(Event::KeyDown(Key::Thrust)).unwrap();
    frontplane.tick(10).unwrap();
    frontplane.event(Event::KeyDown(Key::Pause)).unwrap();
    let before = frontplane.snapshot().unwrap();

    frontplane.tick(30).unwrap();
    let after = frontplane.snapshot().unwrap();

    assert_eq!(&after.bytes[16..40], &before.bytes[16..40]);
    assert_eq!(&after.bytes[64..168], &before.bytes[64..168]);
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
