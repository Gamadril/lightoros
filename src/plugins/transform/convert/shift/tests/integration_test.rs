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
    assert_eq!(plugin_info.name, "ConvertShiftTransform");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Transform);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_transform_convert_shift");
}

#[test]
fn test_create() {
    let config = json!({
        "amount": 10
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}

#[test]
fn test_shift_left() {
    let amount: i32 = -4;
    let config = json!({ "amount": amount });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let plugin = plugin.unwrap();

    let width_input = 5;
    let height_input = 3;

    let size = (width_input - 1) * 2 * (height_input - 1) * 2;
    let mut out: Vec<RGB> = Vec::with_capacity(size);

    for i in 0..size {
        out.push(RGB {
            r: i as u8,
            g: 0xFF,
            b: 0xFF,
        });
    }

    let data = plugin_data!(out, {});
    let result = plugin.transform(&data);
    assert!(result.is_ok());
    let result = result.unwrap();

    let rgb_data = result.rgb;
    assert!(rgb_data.len() == size);

    for i in 0..size {
        let rgb = rgb_data[i];
        let r = (size as i32 + i as i32 - amount) % size as i32;
        assert!(rgb.r == r as u8 && rgb.g == 0xFF && rgb.b == 0xFF);
    }
}

#[test]
fn test_shift_right() {
    let amount: i32 = 6;
    let config = json!({ "amount": amount });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let plugin = plugin.unwrap();

    let width_input = 5;
    let height_input = 3;

    let size = (width_input - 1) * 2 * (height_input - 1) * 2;
    let mut out: Vec<RGB> = Vec::with_capacity(size);

    for i in 0..size {
        out.push(RGB {
            r: i as u8,
            g: 0xFF,
            b: 0xFF,
        });
    }

    let data = plugin_data!(out, {});
    let result = plugin.transform(&data);
    assert!(result.is_ok());
    let result = result.unwrap();

    let rgb_data = result.rgb;
    assert!(rgb_data.len() == size);

    for i in 0..size {
        let rgb = rgb_data[i];
        let r = (size as i32 + i as i32 - amount) % size as i32;
        assert!(rgb.r == r as u8 && rgb.g == 0xFF && rgb.b == 0xFF);
    }
}
