use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use lightoros_plugin::{PluginInfo, PluginInputTrait, RGB, TraitData};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    files: Vec<String>,
    repeat: bool,
    delay_frame: u64,
    delay_file: u64,
}

struct FilesystemInput {
    config: Config,
    frame_offset: usize,
    file_index: usize,
}

impl FilesystemInput {
    fn new(config: &serde_json::Value) -> FilesystemInput {
        let cfg = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        FilesystemInput {
            config: config,
            frame_offset: 0,
            file_index: 0,
        }
    }
}

impl PluginInputTrait for FilesystemInput {
    fn init(&mut self) -> bool {
        true
    }
    fn get(&mut self) -> Option<TraitData> {
        if self.config.files.len() == 0 {
            eprintln!("File list empty");
            return None;
        }

        if self.file_index == self.config.files.len() {
            self.file_index = 0;
            thread::sleep(Duration::from_millis(self.config.delay_file));
        }

        let ref current_file = self.config.files[self.file_index];
        let path = Path::new(current_file);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Error opening file '{}': {}", current_file, error);
                return None;
            }
        };

        let mut data_in: Vec<u8> = Vec::new();
        match file.read_to_end(&mut data_in) {
            Err(error) => {
                eprintln!("Error reading file '{}': {}", current_file, error);
                return None;
            }
            Ok(_) => {}
        }

        let mut index = self.frame_offset;
        if data_in[index] != 0xC9 || data_in[index + 1] != 0xDA {
            panic!("error: not tpm2 file - start missing");
        }

        let size: u16 = (data_in[index + 2] as u16) << 8 | data_in[index + 3] as u16;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size as usize / 3);
        index = index + 4;
        for _ in 0..(size / 3) {
            data_out.push(RGB {
                r: data_in[index],
                g: data_in[index + 1],
                b: data_in[index + 2],
            });
            index = index + 3;
        }

        if data_in[index] != 0x36 {
            panic!("error: not tpm2 file - frame end missing");
        }

        index += 1;

        if index == data_in.len() {
            self.frame_offset = 0;
            self.file_index = self.file_index + 1;
        } else {
            self.frame_offset = index;
            thread::sleep(Duration::from_millis(self.config.delay_frame));
        }

        let result: TraitData = TraitData {
            rgb: data_out,
            meta: HashMap::new(),
        }; 
        Some(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginInputTrait> {
    let plugin = FilesystemInput::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "FilesystemInputTPM2",
        filename: env!("CARGO_PKG_NAME"),
    }
}
