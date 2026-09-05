fn main() {
    let mut attributes = tauri_build::Attributes::new();
    let e2e_capabilities_enabled = ["CARGO_FEATURE_E2E", "CARGO_FEATURE_E2E_WDIO"]
        .into_iter()
        .any(|name| std::env::var_os(name).is_some());
    if e2e_capabilities_enabled {
        attributes = attributes.capabilities_path_pattern("./capabilities/*.json");
    } else {
        attributes = attributes.capabilities_path_pattern("./capabilities/default.json");
    }
    tauri_build::try_build(attributes).expect("failed to build Tauri application metadata");
}
