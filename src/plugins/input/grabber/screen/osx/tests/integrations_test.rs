use lightoros_plugin_base::*;
use lightoros_plugin_base::input::CreateInputPluginResult;
use serde_json::json;
use dlopen::symbor::Library;
use once_cell::sync::Lazy;
use std::path::PathBuf;

static LIB_PATH: Lazy<PathBuf> = Lazy::new(test_cdylib::build_current_project);

fn load_lib() -> Library {
    let lib_path: PathBuf = LIB_PATH.to_path_buf();
    dlopen::symbor::Library::open(&lib_path).unwrap()
}

fn get_info() -> PluginInfo {
    let lib = load_lib();
    let info_func = unsafe { lib.symbol::<fn() -> PluginInfo>("info").unwrap() };
    info_func()
}

fn call_create(config: &serde_json::Value) -> CreateInputPluginResult {
    let lib = load_lib();
    let create_func = unsafe {
        lib.symbol::<fn(&serde_json::Value) -> CreateInputPluginResult>("create")
            .unwrap()
    };
    create_func(config)
}

#[test]
fn test_get_info() {
    let plugin_info = get_info();
    assert_eq!(plugin_info.name, "OsxScreenGrabberInput");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Input);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_input_grabber_screen_osx");
}

#[test]
fn test_create() {
    let config = json!({
        "screen_index": 0,
        "delay_frame": 100,
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_init() {
    let config = json!({
        "screen_index": 0,
        "delay_frame": 100,
    });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut plugin = plugin.unwrap();
    assert!(plugin.init().is_ok());
    assert!(plugin.get().is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    let plugin = call_create(&config);
    assert!(plugin.is_err());   
}
