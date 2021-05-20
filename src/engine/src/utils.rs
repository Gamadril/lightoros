use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use libloading::{Library, Symbol};

use lightoros_plugin_base::*;


pub fn find_plugin_file(name: &str, folder: &str) -> Result<PathBuf, PluginError> {
    // find plugins
    let paths = match fs::read_dir(folder) {
        Ok(paths) => paths,
        Err(e) => {
            return plugin_err!("Cannot find plugins in folder '{}': {}", folder, e);
        }
    };

    for path in paths {
        let full_path = match path {
            Ok(path) => path,
            Err(err) => return plugin_err!("Error getting plugin file path '{}': {}", name, err),
        };
        let file_name = full_path.file_name();
        let file_name = file_name.to_str().unwrap();

        let is_dylib = file_name.ends_with(std::env::consts::DLL_EXTENSION);

        let is_plugin = file_name.contains("lightoros_input")
            || file_name.contains("lightoros_output")
            || file_name.contains("lightoros_transform");
        if is_plugin && is_dylib {
            let mut libfile_path = current_dir().unwrap();
            libfile_path.push(full_path.path());

            let lib = Library::new(libfile_path.as_path()).unwrap();
            let get_info: Symbol<fn() -> PluginInfo> = match unsafe { lib.get(b"info") } {
                Ok(info) => info,
                Err(_) => continue,
            };
            let info = get_info();

            if info.name == name {
                return Ok(libfile_path);
            }
        }
    }
    plugin_err!("Cannot find plugin '{}' in folder '{}'", name, folder)
}

pub fn get_plugin(name: &str, plugins_folder: &str) -> Result<Library, PluginError> {
    let path = find_plugin_file(name, plugins_folder)?;
    Ok(Library::new(path).unwrap())
}

#[macro_export]
macro_rules! create_input_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<
            fn(&serde_json::Value) -> lightoros_plugin_base::input::CreateInputPluginResult,
        > = unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}

#[macro_export]
macro_rules! create_transform_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<
            fn(&serde_json::Value) -> lightoros_plugin_base::transform::CreateTransformPluginResult,
        > = unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}

#[macro_export]
macro_rules! create_output_plugin {
    ($lib:expr, $config:expr) => {{
        let func: Symbol<
            fn(&serde_json::Value) -> lightoros_plugin_base::output::CreateOutputPluginResult,
        > = unsafe { $lib.get(b"create").unwrap() };
        func($config)
    }};
}