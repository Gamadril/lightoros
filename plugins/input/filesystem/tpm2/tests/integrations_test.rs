use lightoros_input_filesystem_tpm2;

use serde_json::json;

#[test]
fn test_create() {
    let config = json!({
        "files": [
            ""
        ],
        "repeat": false,
        "delay_frame": 100,
        "delay_file": 5000
    });
    let _plugin = lightoros_input_filesystem_tpm2::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_empty_config() {
    let config = json!({
    });
    lightoros_input_filesystem_tpm2::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_missing_config_value() {
    let config = json!({
        "files": [
            ""
        ],
        "repeat": false,
        "delay_file": 5000
    });
    lightoros_input_filesystem_tpm2::create(&config);
}

#[test]
fn test_create_with_empty_files_list() {
    let config = json!({
        "files": [],
        "repeat": false,
        "delay_frame": 100,
        "delay_file": 5000
    });
    let mut plugin = lightoros_input_filesystem_tpm2::create(&config);
    let rgb_data = plugin.get();
    assert!(rgb_data.is_none())
}

#[test]
fn test_get_info() {
    let plugin_info = lightoros_input_filesystem_tpm2::info();
    assert_eq!(plugin_info.name, "FilesystemInputTPM2");
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_input_filesystem_tpm2");
}
