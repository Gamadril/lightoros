use lightoros_plugin_base::*;
use lightoros_plugin_base::output::CreateOutputPluginResult;
use serde_json::json;
use dlopen::symbor::Library;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const PORT: &str = "/dev/tty.usbmodem14101";

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


fn call_create(config: &serde_json::Value) -> CreateOutputPluginResult {
    let lib = load_lib();
    let create_func = unsafe {
        lib.symbol::<fn(&serde_json::Value) -> CreateOutputPluginResult>("create")
            .unwrap()
    };
    create_func(config)
}

#[test]
fn test_get_info() {
    let plugin_info = get_info();
    assert_eq!(plugin_info.name, "SerialOutputTPM2");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Output);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_output_serial_tpm2");
}

#[test]
fn test_create() {
    let config = json!({
        "port": ""
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}

#[test]
fn test_create_with_wrong_dattype_in_config() {
    let config = json!({
        "test": true
    });
    assert!(call_create(&config).is_err());
}

#[test]
fn test_send_to_invalid_port() {
    let config = json!({
        "port": "invalid"
    });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let out: Vec<RGB> = Vec::with_capacity(1);
    let data = plugin_data!(out, {});
    let result = plugin.unwrap().send(&data);
    assert!(result.is_ok());
}

// /dev/tty.usbmodem14201

#[test]
fn test_send_empty_data() {
    let config = json!({ "port": PORT });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let out: Vec<RGB> = Vec::new();
    thread::sleep(Duration::from_millis(1000));
    let data = plugin_data!(out, {});
    let result = plugin.unwrap().send(&data);
    assert!(result.is_ok());
}

#[test]
fn test_send_data_red_dot() {
    let config = json!({ "port": PORT });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut out: Vec<RGB> = Vec::with_capacity(1);
    out.push(RGB { r: 255, g: 0, b: 0 });
    thread::sleep(Duration::from_millis(1000));
    let data = plugin_data!(out, {});
    let result = plugin.unwrap().send(&data);
    assert!(result.is_ok());
}

#[test]
fn test_send_data_blue_dot() {
    let config = json!({ "port": PORT });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut out: Vec<RGB> = Vec::with_capacity(1);
    out.push(RGB { r: 0, g: 0, b: 255 });
    thread::sleep(Duration::from_millis(1000));
    let data = plugin_data!(out, {});
    let result = plugin.unwrap().send(&data);
    assert!(result.is_ok());
}

#[test]
fn test_send_data_green_dot() {
    let config = json!({ "port": PORT });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut out: Vec<RGB> = Vec::with_capacity(1);
    out.push(RGB { r: 0, g: 255, b: 0 });
    thread::sleep(Duration::from_millis(1000));
    let data = plugin_data!(out, {});
    let result = plugin.unwrap().send(&data);
    assert!(result.is_ok());
}
