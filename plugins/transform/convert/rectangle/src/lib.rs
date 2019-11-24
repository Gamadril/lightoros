use serde::Deserialize;

use std::collections::HashMap;

use lightoros_plugin::{PluginInfo, PluginTransformTrait, TraitData, RGB};

#[derive(Deserialize, Debug)]
struct Config {
    drop_corners: bool,
}

struct ConvertRectangleTransform {
    config: Config,
}

impl ConvertRectangleTransform {
    fn new(config: &serde_json::Value) -> ConvertRectangleTransform {
        let cfg: serde_json::Value = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        ConvertRectangleTransform { config: config }
    }
}

impl PluginTransformTrait for ConvertRectangleTransform {
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

        let mut size = src_width * 2 + src_height * 2;
        if !self.config.drop_corners {
            size += 4;
        }
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        let mut from = if self.config.drop_corners { 1 } else { 0 };
        let mut to = if self.config.drop_corners {
            src_width - 1
        } else {
            src_width
        };

        for i in from..to {
            let pixel = &rgb_data[i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        from = if self.config.drop_corners { 1 } else { 0 };
        to = if self.config.drop_corners {
            src_height - 1
        } else {
            src_height
        };

        for i in from..to {
            let pixel = &rgb_data[i * src_width + src_width - 1];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        from = if self.config.drop_corners { 1 } else { 0 };
        to = if self.config.drop_corners {
            src_width - 1
        } else {
            src_width
        };

        for i in (from..to).rev() {
            let pixel = &rgb_data[src_width * (src_height - 1) + i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        from = if self.config.drop_corners { 1 } else { 0 };
        to = if self.config.drop_corners {
            src_height - 1
        } else {
            src_height
        };

        for i in (from..to).rev() {
            let pixel = &rgb_data[i * src_width];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        let result = TraitData {
            rgb: data_out,
            meta: HashMap::new(),
        };
        result
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginTransformTrait> {
    let plugin = ConvertRectangleTransform::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "ConvertRectangleTransform",
        filename: env!("CARGO_PKG_NAME"),
    }
}
