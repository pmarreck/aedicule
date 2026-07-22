use aedicule::{WAT_ABI_IMPORTS, wat_abi_markdown};

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
}
