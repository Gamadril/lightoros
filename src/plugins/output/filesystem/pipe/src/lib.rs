use std::ffi::CString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Error;
use std::os::raw::{c_char, c_int, c_ushort};
use std::path::Path;

use std::ops::Deref;

use lightoros_plugin_base::output::{CreateOutputPluginResult, PluginOutputTrait};
use lightoros_plugin_base::*;
use serde::Deserialize;

const NAME: &str = "FilesystemPipeOutput";
const PROTOCOLS: [&str; 2] = ["tpm2", "raw"];

#[derive(Deserialize, Debug)]
struct Config {
    path: String,
    protocol: String,
}

struct FilesystemPipeOutput {
    config: Config,
    file: Option<File>,
}

impl std::fmt::Display for FilesystemPipeOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl FilesystemPipeOutput {
    fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        if !PROTOCOLS.contains(&config.protocol.as_ref()) {
            return plugin_err!("Unsupported protocol: {}", config.protocol);
        }

        let plugin = FilesystemPipeOutput { config, file: None };
        Ok(Box::new(plugin))
    }
}

impl PluginOutputTrait for FilesystemPipeOutput {
    fn init(&mut self) -> PluginResult<()> {
        let pipe_path = Path::new(&self.config.path);
        if pipe_path.exists() {
            let result = std::fs::remove_file(pipe_path);
            if result.is_err() {
                return plugin_err!(
                    "File '{}' already exists and could not be removed: {}",
                    &self.config.path,
                    result.err().unwrap()
                );
            }
        }

        let path: CString = CString::new(self.config.path.as_bytes()).unwrap();
        let result = unsafe { mkfifo(path.as_ptr(), 0o644) };
        if result != 0 {
            let err = Error::last_os_error();
            return plugin_err!(
                "Failed to create named pipe '{}': {}",
                &self.config.path,
                err
            );
        }

        Ok(())
    }

    fn send(&mut self, data: &TraitData) -> PluginResult<()> {
        if self.file.is_none() {
            // open pipe for writing only will block until the pipe is opened for reading on the other side
            let path = Path::new(&self.config.path);
            let pipe_file = match OpenOptions::new().write(true).open(&path) {
                Ok(file) => file,
                Err(err) => {
                    return plugin_err!("Error opening file '{}': {}", &self.config.path, err);
                }
            };
            self.file = Some(pipe_file);
        }

        let rgb_data = data.rgb.deref();
        let mut file_ref = self.file.as_ref().unwrap();
        let mut out: Vec<u8>;

        if self.config.protocol == "tpm2" {
            let out_size = rgb_data.len() * 3;
            out = Vec::with_capacity(out_size + 5);
            out.push(0xC9);
            out.push(0xDA);
            out.push(((out_size >> 8) & 0xFF) as u8);
            out.push((out_size & 0xFF) as u8);
            for rgb in rgb_data {
                out.push(rgb.g);
                out.push(rgb.r);
                out.push(rgb.b);
            }
            out.push(0x36);
        } else if self.config.protocol == "raw" {
            out = Vec::with_capacity(rgb_data.len());
            for rgb in rgb_data {
                out.push(rgb.r);
                out.push(rgb.g);
                out.push(rgb.b);
            }
        } else {
            return plugin_err!("Unsupported protocol '{}'", self.config.protocol);
        }

        let result = file_ref.write_all(&out);
        if result.is_err() {
            return plugin_err!(
                "Could not write to file '{}': {}",
                &self.config.path,
                result.err().unwrap()
            );
        }

        Ok(())
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateOutputPluginResult {
    FilesystemPipeOutput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Output)
}

extern "C" {
    pub fn mkfifo(pathname: *const c_char, mode: c_ushort) -> c_int;
}
