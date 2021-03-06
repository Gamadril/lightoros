use dlopen::symbor::Library;
use lightoros_plugin_base::output::CreateOutputPluginResult;
use lightoros_plugin_base::*;
use once_cell::sync::Lazy;
use serde_json::json;
use std::path::PathBuf;
use std::{thread, time};

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
    assert_eq!(plugin_info.name, "NetUdpOutputTPM2");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Output);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_output_net_udp_tpm2");
}

#[test]
fn test_create() {
    let config = json!({
        "ip": "127.0.0.1",
        "port": 45000,
        "max_packet_length": 1024,
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
fn test_send_all() {
    let config = json!({
        "ip": "127.0.0.1",
        "port": 65506,
        "max_packet_length": 9216,
    });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut plugin = plugin.unwrap();

    let size = 200 * 2 + 160 * 2;
    let mut out: Vec<RGB> = Vec::with_capacity(size);

    for _i in 0..200 {
        out.push(RGB {
            r: 255,
            g: 0,
            b: 0,
        })
    }
    for _i in 0..160 {
        out.push(RGB {
            r: 0,
            g: 255,
            b: 0,
        })
    }
    for _i in 0..200 {
        out.push(RGB {
            r: 0,
            g: 0,
            b: 255,
        })
    }
    for _i in 0..160 {
        out.push(RGB {
            r: 255,
            g: 255,
            b: 0,
        })
    }
    let data = plugin_data!(out, {});
    plugin.send(&data).unwrap();

    let three_seconds = time::Duration::from_millis(3000);
    thread::sleep(three_seconds);

    let mut out: Vec<RGB> = Vec::with_capacity(size);
    for _i in 0..size {
        out.push(RGB {
            r: 0,
            g: 255,
            b: 0,
        })
    }
    let data = plugin_data!(out, {});
    plugin.send(&data).unwrap();

    let three_seconds = time::Duration::from_millis(3000);
    thread::sleep(three_seconds);

    let mut out: Vec<RGB> = Vec::with_capacity(size);
    for _i in 0..size {
        out.push(RGB {
            r: 0,
            g: 0,
            b: 255,
        })
    }
    let data = plugin_data!(out, {});
    plugin.send(&data).unwrap();
}

