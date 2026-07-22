#[path = "src/wat_abi.rs"]
mod wat_abi;

fn main() {
    println!("cargo::rerun-if-changed=src/wat_abi.rs");
    println!("cargo::rerun-if-changed=WAT_ABI.md");
    println!("cargo::rerun-if-env-changed=AE_UPDATE_WAT_ABI");

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
}
