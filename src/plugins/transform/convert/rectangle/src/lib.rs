use serde::Deserialize;

use std::collections::HashMap;

use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

const NAME: &str = "ConvertRectangleTransform";

#[derive(Deserialize, Debug)]
struct Config {
    drop_corners: bool,
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

        let size = if self.config.drop_corners {
            (src_width - 2) * 2 + (src_height - 2) * 2
        } else {
            src_width * 2 + (src_height - 2) * 2
        };
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        let start = if self.config.drop_corners { 1 } else { 0 };
        let end = src_width - 1;

        for i in start..end {
            let pixel = &rgb_data[i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        let start = if self.config.drop_corners { 1 } else { 0 };
        let end = src_height - 1;

        for i in start..end {
            let pixel = &rgb_data[i * src_width + src_width - 1];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        let start = 1;
        let end = if self.config.drop_corners { src_width - 1 } else { src_width };

        for i in (start..end).rev() {
            let pixel = &rgb_data[src_width * (src_height - 1) + i];
            data_out.push(RGB {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            });
        }

        let start = 1;
        let end = if self.config.drop_corners { src_height - 1 } else { src_height };

        for i in (start..end).rev() {
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
