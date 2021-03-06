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
    assert_eq!(plugin_info.name, "ConvertRectangleTransform");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Transform);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(
        plugin_info.filename,
        "lightoros_transform_convert_rectangle"
    );
}

#[test]
fn test_create() {
    let config = json!({
        "drop_corners": false
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}

#[test]
fn test_with_corners() {
    let config = json!({
        "drop_corners": false
    });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let plugin = plugin.unwrap();

    let width_input = 20;
    let height_input = 16;

    let mut out: Vec<RGB> = Vec::with_capacity(width_input * height_input);

    for y in 0..height_input {
        for x in 0..width_input {
            out.push(RGB {
                r: x as u8,
                g: (100 + y) as u8,
                b: 0xAA,
            });
        }
    }

    let out_copy = out.clone();

    let data = plugin_data!(out, {
            "width" => width_input,
            "height" => height_input,
    });
    let result = plugin.transform(&data);
    assert!(result.is_ok());
    let result = result.unwrap();

    let rgb_data = result.rgb;
    assert!(rgb_data.len() == 68);

    for x in 0..width_input {
        let rgb_in = out_copy[x];
        let rgb_out = rgb_data[x];
        assert!(rgb_in.r == rgb_out.r && rgb_in.g == rgb_out.g && rgb_in.b == rgb_out.b);
    }

    for y in 0..height_input {
        let rgb_in = out_copy[width_input * y + width_input - 1];
        let rgb_out = rgb_data[width_input - 1 + y];
        assert!(rgb_in.r == rgb_out.r && rgb_in.g == rgb_out.g && rgb_in.b == rgb_out.b);
    }

    for x in 0..width_input {
        let rgb_in = out_copy[width_input * height_input - 1 - x];
        let rgb_out = rgb_data[width_input - 1 + height_input - 1 + x];
        assert!(rgb_in.r == rgb_out.r && rgb_in.g == rgb_out.g && rgb_in.b == rgb_out.b);
    }

    for y in 0..height_input {
        let rgb_in = out_copy[width_input * (height_input - 1 - y)];
        let rgb_out = rgb_data[((width_input - 1) * 2 + height_input - 1 + y) % rgb_data.len()];
        assert!(rgb_in.r == rgb_out.r && rgb_in.g == rgb_out.g && rgb_in.b == rgb_out.b);
    }
}
