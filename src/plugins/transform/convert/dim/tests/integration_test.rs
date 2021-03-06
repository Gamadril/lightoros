use dlopen::symbor::Library;
use lightoros_plugin_base::transform::CreateTransformPluginResult;
use lightoros_plugin_base::*;
use once_cell::sync::Lazy;
use serde_json::json;
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

fn call_create(config: &serde_json::Value) -> CreateTransformPluginResult {
    let lib = load_lib();
    let create_func = unsafe {
        lib.symbol::<fn(&serde_json::Value) -> CreateTransformPluginResult>("create")
            .unwrap()
    };
    create_func(config)
}

#[test]
fn test_get_info() {
    let plugin_info = get_info();
    assert_eq!(plugin_info.name, "ConvertDimTransform");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Transform);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_transform_convert_dim");
}

#[test]
fn test_create() {
    let config = json!({
        "brightness": 100
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_invalid_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}

#[test]
fn test_create_with_invalid_config_brightness() {
    let config = json!({
        "brightness": 101
    });
    assert!(call_create(&config).is_err());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}
