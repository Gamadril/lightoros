use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;

pub use self::error::PluginError;
mod error;

#[derive(PartialEq)]
pub enum PluginKind {
    Input,
    Output,
    Transform,
}

pub type PluginResult<T> = Result<T, PluginError>;

#[cfg(feature = "input")]
pub mod input {
    use crate::*;

    pub type CreateInputPluginResult = PluginResult<Box<dyn PluginInputTrait>>;

    pub trait PluginInputTrait: Send + Display {
        fn init(&mut self) -> PluginResult<()>;
        fn get(&mut self) -> PluginResult<TraitData>;
    }
}

#[cfg(feature = "output")]
pub mod output {
    use crate::*;

    pub type CreateOutputPluginResult = PluginResult<Box<dyn PluginOutputTrait>>;

    pub trait PluginOutputTrait: Send + Display {
        fn init(&mut self) -> PluginResult<()>;
        fn send(&mut self, data: &TraitData) -> PluginResult<()>;
    }
}

#[cfg(feature = "transform")]
pub mod transform {
    use crate::*;
    pub type CreateTransformPluginResult = PluginResult<Box<dyn PluginTransformTrait>>;

    pub trait PluginTransformTrait: Send + Display {
        fn transform(&self, data: &TraitData) -> PluginResult<TraitData>;
    }
}

pub struct TraitData {
    pub rgb: Vec<RGB>,
    pub meta: HashMap<String, String>,
}

pub struct PluginInfo {
    pub api_version: u8,
    pub name: &'static str,
    pub kind: PluginKind,
    pub filename: &'static str,
}

impl Display for PluginInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.filename)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Display for RGB {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({},{},{})", self.r, self.g, self.b)
    }
}

pub fn get_meta_value<T: std::str::FromStr>(
    meta: &HashMap<String, String>,
    key: &str,
) -> PluginResult<T> {
    let value_str = match meta.get(key) {
        Some(value) => value,
        None => return plugin_err!("Error getting meta data value for the key '{}'", key),
    };
    match value_str.parse::<T>() {
        Ok(value) => Ok(value),
        Err(_) => plugin_err!(
            "Error parsing meta data value '{}' for the key '{}",
            value_str,
            key
        ),
    }
}

pub struct Logger {
    name: String,
}

pub trait LoggerTrait {
    fn error(&self, msg: &str);
    fn debug(&self, msg: &str);
}

impl Logger {
    pub fn new(name: String) -> Logger {
        Logger { name }
    }
}

impl LoggerTrait for Logger {
    fn error(&self, msg: &str) {
        eprintln!("[{}] {}", &self.name, msg);
    }
    #[cfg(debug_assertions)]
    fn debug(&self, msg: &str) {
        println!("[{}] {}", &self.name, msg);
    }

    #[cfg(not(debug_assertions))]
    fn debug(&self, _msg: &str) {}
}

#[macro_export]
macro_rules! get_u32 {
    ($vec: expr, $offset: expr) => {{
        let result = u32::from($vec[$offset]) << 24
            | u32::from($vec[1 + $offset]) << 16
            | u32::from($vec[2 + $offset]) << 8
            | u32::from($vec[3 + $offset]);
        result
    }};
}

#[macro_export]
macro_rules! get_u16 {
    ($vec: expr, $offset: expr) => {{
        let result = u16::from($vec[$offset]) << 8 | u16::from($vec[1 + $offset]);
        result
    }};
}

#[macro_export]
macro_rules! plugin_err {
    ($($arg:tt)*) => {
        Err(PluginError::new(format!($($arg)*)));
    }
}

#[macro_export]
macro_rules! plugin_info {
    ($api: expr, $name: expr, $kind: expr) => {
        PluginInfo {
            api_version: $api,
            name: $name,
            kind: $kind,
            filename: env!("CARGO_PKG_NAME"),
        }
    };
}

#[macro_export]
macro_rules! plugin_data {
    ($data:expr, { $($key:expr => $value:expr,)* }) => {
        TraitData {
            rgb: $data,
            meta: [$(($key.to_string(), $value.to_string()),)*].iter().cloned().collect(),
        }
    };

    ($data:expr, { $($key:expr => $value:expr),* }) => {
        TraitData {
            rgb: $data,
            meta: [$(($key.to_string(), $value.to_string()),)*].iter().cloned().collect(),
        }
    };
}

#[macro_export]
macro_rules! plugin_config_or_return {
    ($cfg:expr) => {{
        let config: Config = match serde_json::from_value($cfg.clone()) {
            Ok(config) => config,
            Err(err) => return plugin_err!("Error deserializing configuration: {}", err),
        };
        config
    }};
}

#[macro_export]
macro_rules! print_image {
    ($data: expr, $width: expr, $height: expr, $name: expr) => {{
        let path = $name.to_owned();
        let p = Path::new(&path);
        if !p.exists() {
            let size = $width as usize * $height as usize;
            let mut buffer: Vec<u8> = Vec::with_capacity(size * 3);
            for i in 0..size {
                let pixel: &RGB = &$data[i];
                buffer.push(pixel.r);
                buffer.push(pixel.g);
                buffer.push(pixel.b);
            }
            image::save_buffer(
                path,
                buffer.as_ref(),
                $width as u32,
                $height as u32,
                image::ColorType::Rgb8,
            ).unwrap();
        }
    }}
}
