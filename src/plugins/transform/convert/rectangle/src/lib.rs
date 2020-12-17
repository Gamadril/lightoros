use serde::Deserialize;

use std::collections::HashMap;

use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

const NAME: &str = "ConvertRectangleTransform";

#[derive(Deserialize, Debug)]
struct Config {
    add_corners: bool,
}

struct ConvertRectangleTransform {
    config: Config,
}

impl std::fmt::Display for ConvertRectangleTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl ConvertRectangleTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = ConvertRectangleTransform { config };
        Ok(Box::new(plugin))
    }
}

impl PluginTransformTrait for ConvertRectangleTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;
        let meta: &HashMap<String, String> = &data.meta;

        let src_width: usize = get_meta_value(meta, "width")?;
        let src_height: usize = get_meta_value(meta, "height")?;

        let mut size = src_width * 2 + src_height * 2;
        if self.config.add_corners {
            size += 4;
        }
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        if self.config.add_corners {
            let pixel = &rgb_data[0];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        for i in 0..src_width {
            let pixel = &rgb_data[i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        if self.config.add_corners {
            let pixel = &rgb_data[src_width - 1];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        for i in 0..src_height {
            let pixel = &rgb_data[i * src_width + src_width - 1];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        if self.config.add_corners {
            let pixel = &rgb_data[src_width * src_height - 1];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        for i in (0..src_width).rev() {
            let pixel = &rgb_data[src_width * (src_height - 1) + i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        if self.config.add_corners {
            let pixel = &rgb_data[src_width * (src_height - 1)];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        for i in (0..src_height).rev() {
            let pixel = &rgb_data[i * src_width];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        let result = plugin_data!(data_out, {});
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    ConvertRectangleTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
