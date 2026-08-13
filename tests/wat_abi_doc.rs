use aedicule::{
    ABI_MAJOR, ABI_MINOR, Frontplane, LLM_GUIDE_VERSION, Limits, WAT_ABI_IMPORTS,
    WAT_ABI_PROPOSALS, abi_reference_html, guide_for_llms_markdown, wat_abi_conformance_module,
    wat_abi_markdown,
};
use std::collections::HashSet;

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
        checked_in.contains(&format!("# Aedicule WAT ABI v{ABI_MAJOR}.{ABI_MINOR}")),
        "the generated reference advertises the current host minor"
    );
    assert!(
        checked_in.contains("## Deterministic RandomZ v1"),
        "ABI v0.9 documents the deterministic random capability"
    );
    assert!(
        checked_in.contains("AE_action"),
        "generated ABI reference documents configure-time standalone actions"
    );
    assert!(
        checked_in.contains("| 16 | Motion gesture |"),
        "generated ABI reference documents the shake gesture event kind"
    );
    assert!(
        checked_in.contains("AE_motion_event"),
        "generated ABI reference documents the six-axis sample export"
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

#[test]
fn checked_in_web_reference_separates_callable_and_proposed_capabilities() {
    let checked_in = std::fs::read_to_string("packaging/web/abi.html")
        .expect("the build-generated Web ABI reference exists");

    assert_eq!(checked_in, abi_reference_html());
    assert!(checked_in.contains("id=\"current\""));
    assert!(checked_in.contains("id=\"proposed\""));
    assert!(checked_in.contains("AE_random_v1_normal"));
    assert!(checked_in.contains("data-abi-status=\"not-callable\""));
    for import in WAT_ABI_IMPORTS {
        assert!(checked_in.contains(import.name));
    }
    let mut slugs = HashSet::new();
    for proposal in WAT_ABI_PROPOSALS {
        assert!(
            slugs.insert(proposal.slug),
            "duplicate proposal slug {}",
            proposal.slug
        );
        let card = format!("id=\"{}\" data-abi-status=\"not-callable\"", proposal.slug);
        assert!(checked_in.contains(&card));
    }
}

#[test]
fn declared_current_imports_link_at_the_documented_value_types() {
    Frontplane::from_wat(&wat_abi_conformance_module(), Limits::default())
        .expect("the complete generated current ABI module links");
}
