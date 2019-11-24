use lightoros_input_extra_lua;
use lightoros_plugin::RGB;
use serde_json::json;

#[test]
fn test_get_info() {
    let plugin_info = lightoros_input_extra_lua::info();
    assert_eq!(plugin_info.name, "ExtraInputLua");
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_input_extra_lua");
}

#[test]
fn test_create() {
    let config = json!({
        "source_folder": "",
        "on_start_effect": {
            "name": "",
            "duration": 0
        },
        "screen": {
            "width": 0,
            "height": 0
        }
    });
    let _plugin = lightoros_input_extra_lua::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_empty_config() {
    let config = json!({
    });
    lightoros_input_extra_lua::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_wrong_dattype_in_config() {
    let config = json!({
        "source_folder": 0,
        "on_start_effect": {
            "name": "",
            "duration": 0
        },
        "screen": {
            "width": 0,
            "height": 0
        }
    });
    lightoros_input_extra_lua::create(&config);
}

#[test]
fn test_create_with_init() {
    let config = json!({
        "source_folder": "",
        "on_start_effect": {
            "name": "",
            "duration": 0
        },
        "screen": {
            "width": 0,
            "height": 0
        }
    });
    let mut plugin = lightoros_input_extra_lua::create(&config);
    plugin.init();
    plugin.get();
}
