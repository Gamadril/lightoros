use lightoros_plugin::{PluginInfo, PluginTransformTrait, TraitData, RGB};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Config {
    width: usize,
    height: usize,
}

struct ResizeImageTransform {
    config: Config,
}

impl ResizeImageTransform {
    fn new(config: &serde_json::Value) -> ResizeImageTransform {
        let cfg: serde_json::Value = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        ResizeImageTransform { config: config }
    }
}

impl PluginTransformTrait for ResizeImageTransform {
    fn transform(&self, data: &TraitData) -> TraitData {
        let rgb_data = &data.rgb;
        let meta: &HashMap<String, String> = &data.meta;

        let width_str: &String = match meta.get("width") {
            Some(value) => value,
            _ => panic!("Missing source image width meta parameter"),
        };
        let src_width = match width_str.parse::<usize>() {
            Ok(number) => number,
            _ => panic!("Cannot parse width meta parameter"),
        };

        let height_str: &String = match meta.get("height") {
            Some(value) => value,
            _ => panic!("Missing source image height meta parameter"),
        };
        let src_height = match height_str.parse::<usize>() {
            Ok(number) => number,
            _ => panic!("Cannot parse height meta parameter"),
        };

        // resize using nearest neighbor
        let width_ratio: f32 = src_width as f32 / self.config.width as f32;
        let height_ratio: f32 = src_height as f32 / self.config.height as f32;

        let size = self.config.width * self.config.height;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let px = (x as f32 * width_ratio).floor() as usize;
                let py = (y as f32 * height_ratio).floor() as usize;
                let pixel: &RGB = &rgb_data[py * src_width + px];
                data_out.push(RGB {
                    r: pixel.r,
                    g: pixel.g,
                    b: pixel.b,
                });
            }
        }

        let result = TraitData {
            rgb: data_out,
            meta: [
                ("width".to_string(), self.config.width.to_string()),
                ("height".to_string(), self.config.height.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        };
        result
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginTransformTrait> {
    let plugin = ResizeImageTransform::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "ResizeImageTransform",
        filename: env!("CARGO_PKG_NAME"),
    }
}
