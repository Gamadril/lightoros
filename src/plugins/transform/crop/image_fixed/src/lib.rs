use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;
use serde::Deserialize;
use std::collections::HashMap;

const NAME: &str = "CropImageFixedTransform";

#[derive(Deserialize, Debug)]
struct Config {
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
}

struct CropImageFixedTransform {
    config: Config,
}

impl std::fmt::Display for CropImageFixedTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl CropImageFixedTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = CropImageFixedTransform { config };
        Ok(Box::new(plugin))
    }
}

impl PluginTransformTrait for CropImageFixedTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;
        let meta: &HashMap<String, String> = &data.meta;

        let src_width: usize = get_meta_value(meta, "width")?;
        let src_height: usize = get_meta_value(meta, "height")?;

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

        let result = plugin_data!(data_out, {
            "width" => width,
            "height" => height,
        });

        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    CropImageFixedTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
