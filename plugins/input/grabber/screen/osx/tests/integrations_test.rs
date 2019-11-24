use lightoros_input_grabber_screen_osx;
use serde_json::json;

#[test]
fn test_get_info() {
    let plugin_info = lightoros_input_grabber_screen_osx::info();
    assert_eq!(plugin_info.name, "OsxScreenGrabberInput");
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_input_grabber_screen_osx");
}

#[test] 
fn test_create() {
    let config = json!({
        "screen_index": 0
    });
    let _plugin = lightoros_input_grabber_screen_osx::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_empty_config() {
    let config = json!({
    });
    lightoros_input_grabber_screen_osx::create(&config);
}

#[test]
fn test_create_with_init() {
    let config = json!({
        "screen_index": 0
    });
    let mut plugin = lightoros_input_grabber_screen_osx::create(&config);
    plugin.init();
    plugin.get();
}