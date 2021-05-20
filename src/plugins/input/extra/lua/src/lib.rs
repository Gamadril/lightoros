use rlua;
use rlua::{Context, FromLua, Function, Lua, Value};

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use std::sync::mpsc;
use std::{thread, time};

use serde::Deserialize;

const NAME: &str = "LuaExtraInput";

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
    duration: u64,
}

#[derive(Deserialize, Debug)]
struct Size {
    width: usize,
    height: usize,
}

#[derive(Deserialize, Debug)]
struct Config {
    source_folder: String,
    on_start_effect: Effect,
    screen: Size,
}

struct LuaExtraInput {
    rx: mpsc::Receiver<Vec<RGB>>,
    tx: mpsc::Sender<Vec<RGB>>,
    config: Config,
    current_effect_index: u32,
    logger: Logger,
}

impl std::fmt::Display for LuaExtraInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

/**
 * Lua scripts are executed in a separate thread since they are executed in a blocking way
*/
impl LuaExtraInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        // LUA thread is sending data to the plugin using a channel
        let (tx, rx) = mpsc::channel();

        let plugin = LuaExtraInput {
            tx: tx,
            rx: rx,
            config: config,
            current_effect_index: 0,
            logger: Logger::new(NAME.to_string()),
        };

        Ok(Box::new(plugin))
    }
}

impl LuaExtraInput {
    fn start_script(&mut self) -> () {
        let thread_tx = self.tx.clone();

        let screen_width = self.config.screen.width;
        let screen_height = self.config.screen.height;

        let mut path = PathBuf::new();
        path.push(&self.config.source_folder);
        path.push([&self.config.on_start_effect.name, "lua"].join("."));
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                self.logger.error(&format!(
                    "Error opening file '{}': {}",
                    &path.display(),
                    error
                ));
                return;
            }
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Err(error) => {
                self.logger.error(&format!(
                    "Error reading file '{}': {}",
                    &path.display(),
                    error
                ));
                return;
            }
            Ok(_) => {}
        }

        let _t = thread::spawn(move || {
            let lua = Lua::new();

            lua.context(|lua_ctx| {
                let globals = lua_ctx.globals();

                let args = lua_ctx.create_table().unwrap();

                let screen = lua_ctx.create_table().unwrap();
                screen.set("width", screen_width).unwrap();
                screen.set("height", screen_height).unwrap();
                globals.set("screen", screen).unwrap();

                let color = lua_ctx.create_table().unwrap();
                let hsv2rgb = lua_ctx
                    .create_function(|_, (hue, saturation, value): (i32, i32, i32)| {
                        let v = value as u8;
                        let mut red: u8 = v;
                        let mut green: u8 = v;
                        let mut blue: u8 = v;

                        if saturation > 0 {
                            let region = hue / 60;
                            let remainder = (hue - (region * 60)) * 256 / 60;

                            let p = ((value * (255 - saturation)) >> 8) as u8;
                            let q = ((value * (255 - ((saturation * remainder) >> 8))) >> 8) as u8;
                            let t = ((value * (255 - ((saturation * (255 - remainder)) >> 8))) >> 8) as u8;

                            match region {
                                0 => {
                                    red = v;
                                    green = t;
                                    blue = p;
                                }
                                1 => {
                                    red = q;
                                    green = v;
                                    blue = p;
                                }
                                2 => {
                                    red = p;
                                    green = v;
                                    blue = t;
                                }
                                3 => {
                                    red = p;
                                    green = q;
                                    blue = v;
                                }
                                4 => {
                                    red = t;
                                    green = p;
                                    blue = v;
                                }
                                _ => {
                                    red = v;
                                    green = p;
                                    blue = q;
                                }
                            }
                        }

                        Ok((
                            rlua::Value::Integer(red as i64),
                            rlua::Value::Integer(green as i64),
                            rlua::Value::Integer(blue as i64),
                        ))
                    })
                    .unwrap();
                color.set("hsv2rgb", hsv2rgb).unwrap();
                globals.set("color", color).unwrap();
                globals.set("args", args).unwrap();

                let core_functions: Vec<(&str, Function)> = vec![
                    (
                        "setScreen",
                        lua_ctx
                            .create_function(move |_: Context, screen: Vec<Vec<LRGB>>| {
                                let width = screen.len();
                                let height = screen[0].len();

                                if width != screen_width || height != screen_height {
                                    return Err(rlua::Error::RuntimeError(format!(
                                        "Screen size does not match. Expected: {}x{}, got: {}x{}",
                                        screen_width, screen_height, width, height
                                    )));
                                }

                                let mut out: Vec<RGB> = Vec::with_capacity(width * height);
                                for i in 0..height {
                                    for j in 0..width {
                                        let dot = screen[j][i];
                                        out.push(RGB {
                                            r: dot.r,
                                            g: dot.g,
                                            b: dot.b,
                                        });
                                    }
                                }

                                thread_tx.send(out).ok();
                                Ok(())
                            })
                            .unwrap(),
                    ),
                    (
                        "sleep",
                        lua_ctx
                            .create_function(|_, ms: u64| {
                                let sleep_time = time::Duration::from_millis(ms);
                                thread::sleep(sleep_time);
                                Ok(())
                            })
                            .unwrap(),
                    ),
                    (
                        "isStopRequested",
                        lua_ctx
                            .create_function(|_, ()| Ok(rlua::Value::Boolean(false)))
                            .unwrap(),
                    ),
                ];
                let api = lua_ctx.create_table_from(core_functions).unwrap();
                globals.set("api", api).unwrap();

                lua_ctx.load(&contents).exec().unwrap();
            });
        });
    }
}

impl PluginInputTrait for LuaExtraInput {
    fn init(&mut self) -> PluginResult<()> {
        self.start_script();
        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        // wait on the channel until the LUA thread calls setScreen() which sends the data over the channel
        let data_out: Vec<RGB> = self.rx.recv().unwrap();

        let result = plugin_data!(data_out, {
            "width" => self.config.screen.width,
            "height" => self.config.screen.height,
        });
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    LuaExtraInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}
