use dlopen::symbor::Library;
use lightoros_plugin_base::*;
use lightoros_plugin_base::transform::CreateTransformPluginResult;
use once_cell::sync::Lazy;
use serde_json::json;
use std::path::PathBuf;

use image::GenericImageView;

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
    assert_eq!(plugin_info.name, "CropImageBlackBorderTransform");
    assert!(plugin_info.kind == lightoros_plugin_base::PluginKind::Transform);
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(
        plugin_info.filename,
        "lightoros_transform_crop_image_black_border"
    );
}

#[test]
fn test_create() {
    let config = json!({
        "threshold": 0xFF
    });
    assert!(call_create(&config).is_ok());
}

#[test]
fn test_create_with_empty_config() {
    let config = json!({});
    assert!(call_create(&config).is_err());
}

#[test]
fn test_detect_borders() {
    let config = json!({
        "threshold": 20
    });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let plugin = plugin.unwrap();

    let width_input = 20;
    let height_input = 10;

    let mut out: Vec<RGB> = Vec::with_capacity(width_input * height_input);

    for y in 0..height_input {
        for _x in 0..width_input {
            if y < 2 || y > height_input - 3 {
                out.push(RGB { r: 0, g: 0, b: 0 });
            } else {
                out.push(RGB {
                    r: 0xFF,
                    g: 0,
                    b: 0,
                });
            }
        }
    }

    let data = plugin_data!(out, {
            "width" => width_input,
            "height" => height_input,
    });
    let result = plugin.transform(&data);
    assert!(result.is_ok());
    let result = result.unwrap();

    let width_output: usize = get_meta_value(&result.meta, "width").unwrap();
    let height_output: usize = get_meta_value(&result.meta, "width").unwrap();

    assert_eq!(width_output, 20);
    assert_eq!(height_output, 6);
}

#[test]
fn test_detect_borders_on_test_image() {
    let img = image::open("tests/test_image.png").unwrap();
    let dimensions = img.dimensions();

    let config = json!({
        "threshold": 20
    });

    let plugin = call_create(&config);
    assert!(plugin.is_ok());
    let plugin = plugin.unwrap();

    let mut out: Vec<RGB> = Vec::with_capacity(dimensions.0 as usize * dimensions.1 as usize);

    let rgba_img = img.as_rgba8().unwrap();
    for y in 0..dimensions.1 {
        for x in 0..dimensions.0 {
            let pixel = rgba_img.get_pixel(x, y);
            out.push(RGB {
                r: pixel[0],
                g: pixel[1],
                b: pixel[2],
            })
        }
    }

    let data = plugin_data!(out, {
        "width" => dimensions.0,
        "height" => dimensions.1,
    });
    
    let result = plugin.transform(&data);
    assert!(result.is_ok());
    let result = result.unwrap();

    let width_output: usize = get_meta_value(&result.meta, "width").unwrap();
    let height_output: usize = get_meta_value(&result.meta, "width").unwrap();

    assert_eq!(width_output, 2048);
    assert_eq!(height_output, 858);

    let mut buffer: Vec<u8> = Vec::with_capacity(width_output * height_output * 3);

    for i in 0..width_output * height_output {
        let pixel: &RGB = &result.rgb[i];
        buffer.push(pixel.r);
        buffer.push(pixel.g);
        buffer.push(pixel.b);
    }

    image::save_buffer(
        "tests/test_image_cropped.png",
        buffer.as_ref(),
        width_output as u32,
        height_output as u32,
        image::ColorType::Rgb8,
    )
    .unwrap()
}
