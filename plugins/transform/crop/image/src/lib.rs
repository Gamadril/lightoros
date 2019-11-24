use lightoros_plugin::{PluginInfo, PluginTransformTrait, TraitData, RGB};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Config {
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
}

struct CropImageTransform {
    config: Config,
}

impl CropImageTransform {
    fn new(config: &serde_json::Value) -> CropImageTransform {
        let cfg: serde_json::Value = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        CropImageTransform { config: config }
    }
}

impl PluginTransformTrait for CropImageTransform {
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

        let width = src_width - self.config.left - self.config.right;
        let height = src_height - self.config.top - self.config.bottom;
        let size = width * height;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        for y in self.config.top..(src_height - self.config.bottom) {
            for x in self.config.left..(src_width - self.config.right) {
                let pixel: &RGB = &rgb_data[src_width * y + x];
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
                ("width".to_string(), width.to_string()),
                ("height".to_string(), height.to_string()),
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
    let plugin = CropImageTransform::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "CropImageTransform",
        filename: env!("CARGO_PKG_NAME"),
    }
}
