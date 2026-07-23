use aedicule::{
    ABI_MINOR, DrawCommand, Frontplane, GEIST_MONO_FAMILY, GEIST_MONO_REGULAR, Limits, TextFont,
    render_svg,
};

const FONT_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_text" (func $legacy_text
		(param i32 i32 i32 f32 f32 f32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_text_font" (func $text_font
		(param i32 i32 i32 f32 f32 f32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "123.45")
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 2)
	(func (export "AE_configure") (result i32) i32.const 0)
	(func (export "AE_init") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_event") (param i32 i32 f32 f32) (result i32) i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 0x080b12ff call $begin drop
		i32.const 1 i32.const 0 i32.const 6
		f32.const 10 f32.const 20 f32.const 14 i32.const -1 i32.const 0
		call $legacy_text drop
		i32.const 2 i32.const 0 i32.const 6
		f32.const 10 f32.const 40 f32.const 14 i32.const -1
		i32.const 1 i32.const 0 call $text_font drop
		call $end)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))"#;

const INTEGER_FONT_WAT: &str = r#"(module
	(import "aedicule.v0" "AE_frame_begin_rgba" (func $begin (param i32) (result i32)))
	(import "aedicule.v0" "AE_text_font_q16" (func $text_font_q16
		(param i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
	(import "aedicule.v0" "AE_frame_end" (func $end (result i32)))
	(memory (export "memory") 1)
	(data (i32.const 0) "1200")
	(func (export "AE_abi_major") (result i32) i32.const 0)
	(func (export "AE_abi_minor") (result i32) i32.const 2)
	(func (export "AE_configure") (result i32) i32.const 0)
	(func (export "AE_init_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_event_i32") (param i32 i32 i32 i32) (result i32) i32.const 0)
	(func (export "AE_tick") (param i32) (result i32) i32.const 0)
	(func (export "AE_render") (result i32)
		i32.const 0x080b12ff call $begin drop
		i32.const 7 i32.const 0 i32.const 4
		i32.const 655360 i32.const 1310720 i32.const 917504 i32.const -1
		i32.const 1 i32.const 1 call $text_font_q16 drop
		call $end)
	(func (export "AE_state_ptr") (result i32) i32.const 64)
	(func (export "AE_state_len") (result i32) i32.const 0)
	(func (export "AE_state_schema") (result i32) i32.const 1))"#;

fn render(source: &str) -> Result<aedicule::FrameOutput, aedicule::FrontplaneError> {
    let mut frontplane = Frontplane::from_wat(source, Limits::default())?;
    frontplane.configure()?;
    frontplane.init(7, 320.0, 240.0)?;
    frontplane.render()
}

#[test]
fn legacy_text_keeps_the_platform_face_and_guests_can_select_geist_mono() {
    let frame = render(FONT_WAT).unwrap();
    assert_eq!(
        frame
            .commands
            .iter()
            .map(|command| match command {
                DrawCommand::Text { font, .. } => *font,
                _ => panic!("font fixture emitted a non-text command"),
            })
            .collect::<Vec<_>>(),
        vec![TextFont::PlatformDefault, TextFont::GeistMonoRegular]
    );

    let svg = render_svg(&frame, 320.0, 240.0);
    assert!(svg.contains("font-family=\"sans-serif\""), "{svg}");
    assert!(
        svg.contains(&format!("font-family=\"{GEIST_MONO_FAMILY}\"")),
        "{svg}"
    );
}

#[test]
fn integer_only_guests_can_render_centered_geist_mono_text() {
    let canonical = wasmprinter::print_bytes(wat::parse_str(INTEGER_FONT_WAT).unwrap()).unwrap();
    assert!(!canonical.contains("f32"), "{canonical}");
    assert!(!canonical.contains("f64"), "{canonical}");

    let frame = render(INTEGER_FONT_WAT).unwrap();
    assert_eq!(
        frame.commands,
        vec![DrawCommand::Text {
            id: 7,
            text: "1200".into(),
            x: 10.0,
            y: 20.0,
            size: 14.0,
            rgba: 0xffff_ffff,
            centered: true,
            font: TextFont::GeistMonoRegular,
        }]
    );
}

#[test]
fn unknown_font_selectors_fail_closed_for_both_coordinate_profiles() {
    for source in [
        FONT_WAT.replace(
            "i32.const 1 i32.const 0 call $text_font",
            "i32.const 99 i32.const 0 call $text_font",
        ),
        INTEGER_FONT_WAT.replace(
            "i32.const 1 i32.const 1 call $text_font_q16",
            "i32.const 99 i32.const 1 call $text_font_q16",
        ),
    ] {
        let error = render(&source).unwrap_err().to_string();
        assert!(error.contains("text font"), "{error}");
    }
}

#[test]
fn bundled_geist_mono_is_the_expected_regular_opentype_payload() {
    assert_eq!(&GEIST_MONO_REGULAR[..4], &[0, 1, 0, 0]);
    let encoded_family = GEIST_MONO_FAMILY
        .encode_utf16()
        .flat_map(u16::to_be_bytes)
        .collect::<Vec<_>>();
    assert!(
        GEIST_MONO_REGULAR
            .windows(encoded_family.len())
            .any(|window| window == encoded_family)
    );
}

#[test]
fn abi_minor_accepts_older_guests_but_rejects_unknown_future_contracts() {
    assert!(render(&FONT_WAT.replacen("i32.const 2", "i32.const 0", 1)).is_ok());
    for minor in [-1, ABI_MINOR + 1] {
        let source = FONT_WAT.replacen("i32.const 2", &format!("i32.const {minor}"), 1);
        let error = Frontplane::from_wat(&source, Limits::default())
            .unwrap_err()
            .to_string();
        assert!(error.contains("ABI minor"), "{error}");
    }
}

#[test]
fn every_delivery_family_carries_the_font_license_and_provenance_is_pinned() {
    let flake = std::fs::read_to_string("flake.nix").unwrap();
    for destination in [
        "$out/share/licenses/aedicule/GeistMono-OFL.txt",
        "$out/ThirdPartyLicenses/GeistMono-OFL.txt",
        "$app/Contents/Resources/ThirdPartyLicenses/GeistMono-OFL.txt",
    ] {
        assert!(flake.contains(destination), "missing {destination}");
    }
    let source = include_str!("../assets/fonts/SOURCE.md");
    assert!(source.contains("10dc7658f13c38a474cde201bb09a4617267545b"));
    assert!(source.contains("5a0de4b3d54ab272f76a1d8c84b7fb24c67bbec6591d5300e61c7bc10094b6c8"));
    assert!(include_str!("../assets/fonts/OFL.txt").contains("SIL OPEN FONT LICENSE Version 1.1"));
}
