use lightoros_plugin_base::transform::{CreateTransformPluginResult, PluginTransformTrait};
use lightoros_plugin_base::*;

const NAME: &str = "ConvertReverseTransform";

struct ConvertReverseTransform {
}

impl std::fmt::Display for ConvertReverseTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl ConvertReverseTransform {
    fn create(_config: &serde_json::Value) -> CreateTransformPluginResult {
        let plugin = ConvertReverseTransform { };
        Ok(Box::new(plugin))
    }
}

impl PluginTransformTrait for ConvertReverseTransform {
    fn transform(&self, data: &TraitData) -> PluginResult<TraitData> {
        let mut rgb_data = data.rgb.clone();
        let mut last = rgb_data.len() - 1;
        let mut first = 1;

        while first < last {
            let f = rgb_data[first];
            let l = rgb_data[last];
            rgb_data[last] = f;
            rgb_data[first] = l;
            first += 1;
            last -= 1;
        }

        let result = plugin_data!(rgb_data, {});

        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateTransformPluginResult {
    ConvertReverseTransform::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Transform)
}
