use dlopen::symbor::Library;
use lightoros_plugin_base::output::CreateOutputPluginResult;
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
    assert_eq!(plugin_info.name, "FilesystemPipeOutput");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Output);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_output_filesystem_pipe");
}

#[test]
fn test_create() {
    let config = json!({
        "path": "./pipe_file",
        "protocol": "tpm2"
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    let plugin = call_create(&config);
    assert!(plugin.is_err());   
}

#[test]
fn test_create_with_init() {
    let config = json!({
        "path": "./pipe_file_1",
        "protocol": "tpm2"
    });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut plugin = plugin.unwrap();
    let res = plugin.init();
    assert!(res.is_ok());
    std::fs::remove_file("./pipe_file_1").unwrap();
}

#[test]
fn test_create_with_send() {
    let config = json!({
        "path": "./pipe_file_2",
        "protocol": "tpm2"
    });
    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let mut plugin = plugin.unwrap();
    assert!(plugin.init().is_ok());
    
    let size = 10;
    let mut out: Vec<RGB> = Vec::with_capacity(size);
    for _i in 0..size {
        out.push(RGB {
            r: 255,
            g: 0,
            b: 0,
        })
    }
    let data = plugin_data!(out, {});
    // send will blocks, create a reader in a separate thread
    std::thread::spawn(|| {
        let _file = std::fs::File::open("./pipe_file_2").unwrap();
        // keep open for 1 second
        std::thread::sleep(std::time::Duration::from_millis(1000));
    });
    let res = plugin.send(&data);
    assert!(res.is_ok());
    std::fs::remove_file("./pipe_file_2").unwrap();
}