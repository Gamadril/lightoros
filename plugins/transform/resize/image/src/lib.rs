use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;
use serde::Deserialize;
use std::collections::HashMap;

const NAME: &str = "ResizeImageTransform";

#[derive(Deserialize, Debug)]
struct Config {
    width: usize,
    height: usize,
}

struct ResizeImageTransform {
    config: Config,
}

impl std::fmt::Display for ResizeImageTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl ResizeImageTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = ResizeImageTransform { config };
        Ok(Box::new(plugin))
    }
}

impl PluginTransformTrait for ResizeImageTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;
        let meta: &HashMap<String, String> = &data.meta;

        let src_width: usize = get_meta_value(meta, "width")?;
        let src_height: usize = get_meta_value(meta, "height")?;

        // resize using nearest neighbour
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

        let result = plugin_data!(data_out, {
            "width" => self.config.width,
            "height" => self.config.height,
        });

        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    ResizeImageTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
