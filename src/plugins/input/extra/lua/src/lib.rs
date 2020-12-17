use rlua;
use rlua::{Function, Lua, Value, Context, FromLua};

use lightoros_plugin::{PluginInfo, PluginInputTrait, RGB, TraitData, PluginKind};
use std::{thread, time};
use std::sync::mpsc;
use std::collections::HashMap;

use serde::{Deserialize};

#[derive(Copy, Clone, Debug)]
pub struct LRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl<'lua> FromLua<'lua> for LRGB {
    fn from_lua(value: Value<'lua>, _: Context<'lua>) -> rlua::Result<Self> {
            if let Value::Table(table) = value {
                Ok(LRGB {
                    r: table.raw_get(1).unwrap(),
                    g: table.raw_get(2).unwrap(),
                    b: table.raw_get(3).unwrap(),
                })
            } else {
                Err(rlua::Error::FromLuaConversionError {
                    from: "LUA type",
                    to: "LRGB",
                    message: Some("expected table".to_string()),
                })
            }
    }
}

#[derive(Deserialize, Debug)]
struct Effect {
    name: String,
    duration: u64
}

#[derive(Deserialize, Debug)]
struct Size {
    width: u32,
    height: u32
}

#[derive(Deserialize, Debug)]
struct Config {
    source_folder: String,
    on_start_effect: Effect,
    screen: Size
}

struct EffectEngineInput {
    rx: mpsc::Receiver<TraitData>,
    tx: mpsc::Sender<TraitData>,
    config: Config,
    current_effect_index: u32
}

/**
 * Lua scripts are executed in a separate thread since they are executed in a blocking way
*/
impl EffectEngineInput {
    fn new(config: serde_json::Value) -> EffectEngineInput {
        let config = match serde_json::from_value(config) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        // LUA thread is sending data to the plugin using a channel
        let (tx, rx) = mpsc::channel();

        EffectEngineInput {
            tx: tx,
            rx: rx,
            config: config,
            current_effect_index: 0
        }
    }

    fn start_script(&mut self) -> () {

        let thread_tx = self.tx.clone();

        let _t = thread::spawn(move || {
            let lua = Lua::new();

            lua.context(|lua_ctx| {
                let globals = lua_ctx.globals();

                let color = lua_ctx.create_table().unwrap();
                color.set(1, 255).unwrap();
                color.set(2, 0).unwrap();
                color.set(3, 0).unwrap();

                let args = lua_ctx.create_table().unwrap();
                args.set("color", color).unwrap();
                globals.set("args", args).unwrap();

                let core_functions: Vec<(&str, Function)> = vec![
                    (
                        "setScreen",
                        lua_ctx.create_function(move |_: Context, screen: Vec<Vec<LRGB>>| {
                            let width = screen.len();
                            let height = screen[0].len();
                            let mut out: Vec<RGB> = Vec::with_capacity(width * height);

                            for column in &screen {
                                for dot in column {
                                    out.push(RGB {
                                        r: dot.r,
                                        g: dot.g,
                                        b: dot.b,
                                    });
                                }
                            }
                            
                            let result = TraitData {
                                rgb: out,
                                meta: HashMap::new()
                            };
                            thread_tx.send(result).unwrap();
                            Ok(())
                        }).unwrap(),
                    ),
                    (
                        "sleep",
                        lua_ctx.create_function(|_, ms: u64| {
                            let sleep_time = time::Duration::from_millis(ms);
                            thread::sleep(sleep_time);
                            Ok(())
                        }).unwrap(),
                    ),
                    (
                        "isStopRequested",
                        lua_ctx.create_function(|_, ()| {
                            Ok(rlua::Value::Boolean(false))
                        }).unwrap(),
                    )
                ];
                
                let api = lua_ctx.create_table_from(core_functions).unwrap();
                globals.set("api", api).unwrap();

                lua_ctx.load(
                    r#"
                    local screen = {}
                    for x = 1, 10 do
                        screen[x] = {}
                        for y = 1, 5 do
                            screen[x][y] = { 10, 20, 200 }
                        end
                    end
                    api.sleep(1000)
                    api.setScreen(screen)
                    "#,
                )
                .exec().unwrap();
            });
        });
    }
}

impl PluginInputTrait for EffectEngineInput {
    fn init(&mut self) -> bool {
        self.start_script(); 

        true
    }

    fn get(&mut self) -> Option<TraitData> {
        // wait on the channel until the LUA thread calls setScreen() which sends the data over the channel
        let out: TraitData = self.rx.recv().unwrap();

        Some(out)
    }
}

#[no_mangle]
pub fn create(config: serde_json::Value) -> Box<dyn PluginInputTrait> {
    let plugin = EffectEngineInput::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "ExtraInputLua",
        kind: PluginKind::Input,
        filename: env!("CARGO_PKG_NAME"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
