use serde::Deserialize;

use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

const NAME: &str = "ConvertDimTransform";

#[derive(Deserialize, Debug)]
struct Config {
    brightness: u8,
}

struct ConvertDimTransform {
    config: Config,
}

impl std::fmt::Display for ConvertDimTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl ConvertDimTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        if config.brightness > 100 {
            return plugin_err!("Invalid config value 'brightness': {}. Valid range: [0-100]", config.brightness)
        }

        let plugin = ConvertDimTransform { config };
        Ok(Box::new(plugin))
    }

    fn dim(&self, value: u8) -> u8 {
        let v = value as u16 * self.config.brightness as u16;
        return (v / 100) as u8
    }
}

impl PluginTransformTrait for ConvertDimTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;
        let out_size = rgb_data.len();

        let mut data_out: Vec<RGB> = Vec::with_capacity(out_size);

        for i in 0..out_size {
            let mut rgb = rgb_data[i];
            rgb.r = self.dim(rgb.r);
            rgb.g = self.dim(rgb.g);
            rgb.b = self.dim(rgb.b);
            data_out.push(rgb)
        }

        let result = plugin_data!(data_out, {});
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    ConvertDimTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
