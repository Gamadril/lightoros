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
    assert_eq!(plugin_info.name, "LuaExtraInput");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Input);
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
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}
