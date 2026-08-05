use aedicule::{
    ABI_MAJOR, ABI_MINOR, LLM_GUIDE_VERSION, WAT_ABI_IMPORTS, guide_for_llms_markdown,
    wat_abi_markdown,
};

#[test]
fn checked_in_wat_abi_reference_is_the_generated_canonical_document() {
    let checked_in = std::fs::read_to_string("WAT_ABI.md").expect("WAT_ABI.md exists");

    assert_eq!(checked_in, wat_abi_markdown());
    for import in WAT_ABI_IMPORTS {
        assert!(
            checked_in.contains(import.name),
            "generated ABI reference documents {}",
            import.name
        );
    }
    assert!(
        checked_in.contains("AE_DISPLAY_REFRESH_RATE"),
        "generated ABI reference documents the host display-rate override"
    );
    assert!(
        checked_in.contains("60000/1001"),
        "generated ABI reference preserves the exact nominal-rate spelling"
    );
    assert!(
        checked_in.contains(
            "Pointer button IDs are `1` primary/left, `2` secondary/right, and `3` middle/wheel-click."
        ),
        "generated ABI reference assigns all stable cross-platform pointer button IDs"
    );
    assert!(
        checked_in.contains("| 10 | Pointer scroll |"),
        "generated ABI reference documents two-axis pointer scrolling"
    );
    assert!(
        checked_in.contains("| 6 | Device change |"),
        "generated ABI reference renames kind 6 to the device-change event"
    );
    assert!(
        checked_in.contains("Device-class flag bit `0` is set when the primary pointer is coarse"),
        "generated ABI reference defines device-class flag bit 0"
    );
    assert!(
        checked_in.contains("# Aedicule WAT ABI v0.6"),
        "device-class flags are a new capability, so the host minor advertises v0.6"
    );
    assert!(
        checked_in.contains("`12` F1, `13` W, `14` A, and `15` D"),
        "generated ABI reference assigns append-only W/A/D physical-key IDs"
    );
    assert!(
        checked_in.contains("AE_external_link")
            && checked_in.contains("AE_external_link_place_q16")
            && checked_in.contains("HTTPS")
            && checked_in.contains("noopener"),
        "generated ABI reference documents bounded user-activated external links"
    );
}

#[test]
fn checked_in_llm_guide_is_generated_versioned_and_complete_without_host_source() {
    let checked_in =
        std::fs::read_to_string("GUIDE_FOR_LLMS.md").expect("GUIDE_FOR_LLMS.md exists");

    assert_eq!(checked_in, guide_for_llms_markdown());
    let abi_heading = format!("\n### Aedicule WAT ABI v{ABI_MAJOR}.{ABI_MINOR}\n");
    let wrong_heading = format!("\n## Aedicule WAT ABI v{ABI_MAJOR}.{ABI_MINOR}\n");
    assert!(checked_in.contains(&abi_heading));
    assert!(!checked_in.contains(&wrong_heading));
    assert!(
        !checked_in.lines().any(|line| line.ends_with(' ')),
        "the generated guide must not rely on trailing-space Markdown breaks"
    );
    assert!(
        checked_in.ends_with('\n') && !checked_in.ends_with("\n\n"),
        "the generated guide must end with exactly one newline"
    );
    assert!(checked_in.contains(&format!("Guide version: `{LLM_GUIDE_VERSION}`")));
    assert!(
        checked_in.contains("https://github.com/pmarreck/aedicule/blob/yolo/GUIDE_FOR_LLMS.md")
    );
    assert!(checked_in.contains("## Available now"));
    assert!(checked_in.contains("## Proposed—not callable yet"));
    assert!(checked_in.contains("## Common WAT and LLM mistakes"));
    assert!(checked_in.contains("## Recommended toolchain"));
    assert!(checked_in.contains("### Host-scheduled pause"));
    assert!(checked_in.contains("### External links"));
    assert!(checked_in.contains("does not alter or deduplicate its raw input lifecycle"));
    assert!(checked_in.contains("Comment intent, units, invariants, and state layout"));
    for import in WAT_ABI_IMPORTS {
        assert!(
            checked_in.contains(import.name),
            "generated LLM guide documents current import {}",
            import.name
        );
    }
    let readme = std::fs::read_to_string("README.md").expect("README.md exists");
    assert!(
        readme.contains("[Guide for LLMs](GUIDE_FOR_LLMS.md)"),
        "the canonical guide is discoverable from the repository landing page"
    );
}
