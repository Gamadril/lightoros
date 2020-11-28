use image;
use lightoros_plugin::TraitData;
use lightoros_plugin::RGB;
use lightoros_transform_crop_image_black_border;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_get_info() {
    let plugin_info = lightoros_transform_crop_image_black_border::info();
    assert_eq!(plugin_info.name, "CropImageBlackBorderTransform");
    assert!(plugin_info.kind == lightoros_plugin::PluginKind::Transform);
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
    let _plugin = lightoros_transform_crop_image_black_border::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_empty_config() {
    let config = json!({});
    lightoros_transform_crop_image_black_border::create(&config);
}

#[test]
fn test_detect_borders() {
    let config = json!({
        "threshold": 0xFF
    });
    let plugin = lightoros_transform_crop_image_black_border::create(&config);
    let img = image::open("tests/test_image.png").unwrap();
    let rgb_image = img.as_rgb8().unwrap();
    let (width, height) = rgb_image.dimensions();
    let mut out: Vec<RGB> = Vec::with_capacity(width as usize * height as usize);
    for pixel in rgb_image.pixels() {
        out.push(RGB {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2],
        });
    }
    let data = TraitData {
        rgb: out,
        meta: HashMap::new(),
    };
    let result = plugin.transform(&data);

    let width_str: &String = match result.meta.get("width") {
        Some(value) => value,
        _ => panic!("Missing source image width meta parameter"),
    };
    let src_width = match width_str.parse::<usize>() {
        Ok(number) => number,
        _ => panic!("Cannot parse width meta parameter"),
    };

    let height_str: &String = match result.meta.get("height") {
        Some(value) => value,
        _ => panic!("Missing source image height meta parameter"),
    };
    let src_height = match height_str.parse::<usize>() {
        Ok(number) => number,
        _ => panic!("Cannot parse height meta parameter"),
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(src_width * src_height * 3);

    for i in 0..src_width * src_height {
        let pixel: &RGB = &result.rgb[i];
        buffer.push(pixel.r);
        buffer.push(pixel.g);
        buffer.push(pixel.b);
    }

    image::save_buffer(
        "image_cropped.png",
        buffer.as_ref(),
        src_width as u32,
        src_height as u32,
        image::RGB(8),
    )
    .unwrap()
}
