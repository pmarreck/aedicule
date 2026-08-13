#[allow(dead_code)]
#[path = "src/wat_abi.rs"]
mod wat_abi;

fn main() {
    println!("cargo::rerun-if-changed=src/wat_abi.rs");
    println!("cargo::rerun-if-changed=WAT_ABI.md");
    println!("cargo::rerun-if-changed=GUIDE_FOR_LLMS.template.md");
    println!("cargo::rerun-if-changed=GUIDE_FOR_LLMS.md");
    println!("cargo::rerun-if-changed=packaging/web/abi.html");
    println!("cargo::rerun-if-env-changed=AE_UPDATE_WAT_ABI");
    println!("cargo::rerun-if-env-changed=AE_UPDATE_LLM_GUIDE");
    println!("cargo::rerun-if-env-changed=AE_UPDATE_ABI_HTML");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let resource = "packaging/windows/aedicule.rc";
        println!("cargo::rerun-if-changed={resource}");
        println!("cargo::rerun-if-changed=assets/icons/Aedicule.ico");
        embed_resource::compile(resource, embed_resource::NONE)
            .manifest_optional()
            .expect("Aedicule's Windows icon resource must compile");
    }

    let checked_in = std::fs::read_to_string("WAT_ABI.md")
        .expect("WAT_ABI.md must exist for every Aedicule build");
    let generated = wat_abi::wat_abi_markdown();
    if std::env::var_os("AE_UPDATE_WAT_ABI").is_none() {
        assert_eq!(
            checked_in, generated,
            "WAT_ABI.md is stale; run `AE_UPDATE_WAT_ABI=1 cargo run --bin generate-wat-abi > WAT_ABI.md` and check in the result"
        );
    }

    let checked_in_guide = std::fs::read_to_string("GUIDE_FOR_LLMS.md")
        .expect("GUIDE_FOR_LLMS.md must exist for every Aedicule build");
    let generated_guide = wat_abi::guide_for_llms_markdown();
    if std::env::var_os("AE_UPDATE_LLM_GUIDE").is_none() {
        assert_eq!(
            checked_in_guide, generated_guide,
            "GUIDE_FOR_LLMS.md is stale; run `AE_UPDATE_LLM_GUIDE=1 cargo run --bin generate-llm-guide > GUIDE_FOR_LLMS.md` and check in the result"
        );
    }

    let checked_in_html = std::fs::read_to_string("packaging/web/abi.html")
        .expect("packaging/web/abi.html must exist for every Aedicule build");
    let generated_html = wat_abi::abi_reference_html();
    if std::env::var_os("AE_UPDATE_ABI_HTML").is_none() {
        assert_eq!(
            checked_in_html, generated_html,
            "packaging/web/abi.html is stale; run `AE_UPDATE_ABI_HTML=1 cargo run --bin generate-abi-html > packaging/web/abi.html` and check in the result"
        );
    }
}
