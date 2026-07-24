use aedicule::{
    ExternalLink, ExternalLinkPlacement, ExternalLinkRequest, Frontplane, Limits, Rect, UiSnapshot,
};

const EXTERNAL_LINK_WAT: &str = include_str!("fixtures/external_link.wat");

fn configured_link(wat: &str) -> Result<Frontplane, String> {
    let mut frontplane =
        Frontplane::from_wat(wat, Limits::default()).map_err(|error| error.to_string())?;
    frontplane.configure().map_err(|error| error.to_string())?;
    frontplane
        .init(0x5eed, 640.0, 480.0)
        .map_err(|error| error.to_string())?;
    Ok(frontplane)
}

#[test]
fn configured_https_link_becomes_one_atomic_guest_positioned_view() {
    let mut frontplane = configured_link(EXTERNAL_LINK_WAT).unwrap();

    assert_eq!(
        frontplane.metadata().external_links,
        vec![ExternalLink {
            id: 7,
            label: "What is this?".to_owned(),
            url: "https://example.com/readme#ulam".to_owned(),
        }]
    );
    frontplane.render().unwrap();
    assert_eq!(
        frontplane.ui_snapshot(),
        Some(&UiSnapshot {
            revision: 1,
            control_panels: vec![aedicule::ControlPanel {
                id: 5,
                bounds: Rect {
                    x: 10.0,
                    y: 20.0,
                    width: 300.0,
                    height: 48.0,
                },
                rgba: 0x101820e8,
            }],
            sliders: Vec::new(),
            buttons: Vec::new(),
            external_links: vec![ExternalLinkPlacement {
                id: 7,
                panel_id: 5,
                bounds: Rect {
                    x: 16.0,
                    y: 24.0,
                    width: 280.0,
                    height: 32.0,
                },
            }],
        })
    );
    assert_eq!(
        frontplane.external_link_request(1, 7).unwrap(),
        ExternalLinkRequest {
            revision: 1,
            id: 7,
            label: "What is this?".to_owned(),
            url: "https://example.com/readme#ulam".to_owned(),
        }
    );
}

#[test]
fn external_link_declarations_reject_ambient_or_ambiguous_authority() {
    for invalid_url in [
        "http://example.com/readme",
        "javascript:alert(1)",
        "file:///etc/passwd",
        "https://user:secret@example.com/readme",
        "https://example.com\\@attacker.invalid/",
        "https://",
    ] {
        let wat = EXTERNAL_LINK_WAT
            .replace("https://example.com/readme#ulam", invalid_url)
            .replace(
                "i32.const 32 i32.const 31",
                &format!("i32.const 32 i32.const {}", invalid_url.len()),
            );
        assert!(
            configured_link(&wat).is_err(),
            "admitted unsafe external URL {invalid_url:?}"
        );
    }
}

#[test]
fn external_link_declarations_are_nonempty_bounded_unique_and_configure_only() {
    let empty_label =
        EXTERNAL_LINK_WAT.replace("i32.const 0 i32.const 13", "i32.const 0 i32.const 0");
    assert!(configured_link(&empty_label).is_err());

    let unsupported_flags = EXTERNAL_LINK_WAT.replacen(
        "i32.const 0\n\t\tcall $external_link",
        "i32.const 1\n\t\tcall $external_link",
        1,
    );
    assert!(configured_link(&unsupported_flags).is_err());

    let duplicate = EXTERNAL_LINK_WAT.replace(
		"call $external_link)",
		"call $external_link drop\n\t\ti32.const 7\n\t\ti32.const 0 i32.const 13\n\t\ti32.const 32 i32.const 31\n\t\ti32.const 0\n\t\tcall $external_link)",
	);
    assert!(configured_link(&duplicate).is_err());

    let render_declaration = EXTERNAL_LINK_WAT.replace(
		"global.get $invalid i32.const 1 i32.add call $ui_begin drop",
		"i32.const 7\n\t\ti32.const 0 i32.const 13\n\t\ti32.const 32 i32.const 31\n\t\ti32.const 0\n\t\tcall $external_link drop\n\t\ti32.const 1 call $ui_begin drop",
	);
    let mut frontplane = configured_link(&render_declaration).unwrap();
    assert!(frontplane.render().is_err());
}

#[test]
fn link_placement_rejects_unknown_panels_undeclared_links_flags_and_duplicate_ids() {
    let mutants = [
		EXTERNAL_LINK_WAT.replacen(
			"i32.const 7 i32.const 5",
			"i32.const 7 i32.const 99",
			1,
		),
		EXTERNAL_LINK_WAT.replacen(
			"i32.const 7 i32.const 5",
			"i32.const 99 i32.const 5",
			1,
		),
		EXTERNAL_LINK_WAT.replacen(
			"i32.const 0\n\t\t\tcall $external_link_place",
			"i32.const 1\n\t\t\tcall $external_link_place",
			1,
		),
		EXTERNAL_LINK_WAT.replace(
			"call $external_link_place drop\n\t\t\tcall $ui_end",
			"call $external_link_place drop\n\t\t\ti32.const 7 i32.const 5\n\t\t\ti32.const 1048576 i32.const 1572864\n\t\t\ti32.const 18350080 i32.const 2097152\n\t\t\ti32.const 0\n\t\t\tcall $external_link_place drop\n\t\t\tcall $ui_end",
		),
	];

    for mutant in mutants {
        assert_ne!(mutant, EXTERNAL_LINK_WAT, "mutation must alter the fixture");
        let mut frontplane = configured_link(&mutant).unwrap();
        assert!(frontplane.render().is_err());
        assert!(frontplane.ui_snapshot().is_none());
    }
}

#[test]
fn activation_requires_the_current_accepted_revision_and_placement() {
    let mut frontplane = configured_link(EXTERNAL_LINK_WAT).unwrap();
    frontplane.render().unwrap();

    assert!(frontplane.external_link_request(1, 7).is_ok());
    assert!(frontplane.external_link_request(2, 7).is_err());
    assert!(frontplane.external_link_request(1, 99).is_err());

    frontplane
        .event(aedicule::Event::KeyDown(aedicule::Key::Escape))
        .unwrap();
    assert!(frontplane.render().is_err());
    assert!(frontplane.external_link_request(1, 7).is_ok());
    assert!(frontplane.external_link_request(2, 7).is_err());
}

#[test]
fn external_link_fixture_remains_integer_only() {
    let wasm = wat::parse_str(EXTERNAL_LINK_WAT).unwrap();
    let canonical = wasmprinter::print_bytes(wasm).unwrap();

    assert!(!canonical.contains("f32"), "{canonical}");
    assert!(!canonical.contains("f64"), "{canonical}");
}
