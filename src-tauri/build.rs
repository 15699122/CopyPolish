fn main() {
    let mut attributes = tauri_build::Attributes::new();
    if std::env::var_os("CARGO_FEATURE_E2E").is_some() {
        attributes = attributes.capabilities_path_pattern("./capabilities/*.json");
    } else {
        attributes = attributes.capabilities_path_pattern("./capabilities/default.json");
    }
    tauri_build::try_build(attributes).expect("failed to build Tauri application metadata");
}
