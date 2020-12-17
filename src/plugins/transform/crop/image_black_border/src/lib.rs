use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const NAME: &str = "CropImageBlackBorderTransform";

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    threshold: u8,
}

struct CropImageBlackBorderTransform {
    config: Config,
}

impl std::fmt::Display for CropImageBlackBorderTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl CropImageBlackBorderTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = CropImageBlackBorderTransform { config };
        Ok(Box::new(plugin))
    }

    fn is_black(&self, pixel: &RGB) -> bool {
        pixel.r <= self.config.threshold
            && pixel.g <= self.config.threshold
            && pixel.b <= self.config.threshold
    }
}

impl PluginTransformTrait for CropImageBlackBorderTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;
        let meta: &HashMap<String, String> = &data.meta;
        let src_width: usize = get_meta_value(meta, "width")?;
        let src_height: usize = get_meta_value(meta, "height")?;
        let mut crop_top: i32 = -1;
        let mut crop_bottom: i32 = -1;
        let mut crop_left: i32 = -1;
        let mut crop_right: i32 = -1;

        for y in 0..src_height / 2 {
            let pixel_top: &RGB = &rgb_data[src_width * y + src_width / 2];
            let pixel_bottom: &RGB = &rgb_data[src_width * (src_height - 1 - y) + src_width / 2];

            if !self.is_black(pixel_top) && crop_top < 0 {
                crop_top = y as i32;
            }

            if !self.is_black(pixel_bottom) && crop_bottom < 0 {
                crop_bottom = (src_height - y) as i32;
            }

            if crop_top >= 0 && crop_bottom >= 0 {
                break;
            }
        }

        for x in 0..src_width / 2 {
            let pixel_left: &RGB = &rgb_data[src_width * (src_height / 2) + x];
            let pixel_right: &RGB = &rgb_data[src_width * (src_height / 2) + (src_width - 1 - x)];

            if !self.is_black(pixel_left) && crop_left < 0 {
                crop_left = x as i32;
            }

            if !self.is_black(pixel_right) && crop_right < 0 {
                crop_right = (src_width - x) as i32;
            }

            if crop_left >= 0 && crop_right >= 0 {
                break;
            }
        }

        let width = crop_right - crop_left;
        let height = crop_bottom - crop_top;
        let mut data_out: Vec<RGB> = Vec::with_capacity((width * height) as usize);

        for y in crop_top..crop_bottom {
            for x in crop_left..crop_right {
                let pixel: &RGB = &rgb_data[(width * y + x) as usize];
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
    CropImageBlackBorderTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
