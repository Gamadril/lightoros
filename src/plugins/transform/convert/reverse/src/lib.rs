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
        rgb_data.reverse();
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
