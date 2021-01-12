use serde::Deserialize;

use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

const NAME: &str = "ConvertShiftTransform";

#[derive(Deserialize, Debug)]
struct Config {
    amount: i32,
}

struct ConvertShiftTransform {
    config: Config,
}

impl std::fmt::Display for ConvertShiftTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl ConvertShiftTransform {
    fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = ConvertShiftTransform { config };
        Ok(Box::new(plugin))
    }
}

impl PluginTransformTrait for ConvertShiftTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let rgb_data = &data.rgb;

        let out_size = rgb_data.len();
        let mut data_out: Vec<RGB> = Vec::with_capacity(out_size);

        for i in 0..out_size {
            let idx = i as i32 + out_size as i32 - self.config.amount;
            let idx = idx as usize % out_size;
            let rgb = rgb_data[idx];
            data_out.push(rgb)
        }

        let result = plugin_data!(data_out, {});
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    ConvertShiftTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
