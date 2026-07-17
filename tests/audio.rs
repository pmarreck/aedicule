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
