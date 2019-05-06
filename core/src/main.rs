
use clap::crate_version;
use serde::{Deserialize};
use serde_json::Value;
use clap::{App, Arg};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::thread::JoinHandle;
use std::thread::ThreadId;
use std::time::{Duration, Instant};
use std::fmt;
use std::sync::Arc;
use libloading::{Library, Symbol};
use std::env::current_dir;

use lightoros_plugin::*;

#[derive(Deserialize)]
struct Plugins {
    input: Vec<Value>,
    output: Vec<Value>,
}

#[derive(Deserialize)]
struct Config {
    max_input_inactivity_period: u64,
    plugins: Plugins,
}

struct InputThread {
    priority: u8,
    handle: JoinHandle<()>,
    id: ThreadId,
}

struct OutputThread {
    handle: JoinHandle<()>,
    id: ThreadId,
}

struct InputPipe {
    priority: u8,
    //handle: Option<JoinHandle<()>>,
    //id: ThreadId,
    name: String,
    channel_out: Sender<InputEvent>,
    input: Box<PluginInputTrait>,
    transformations: Vec<Box<PluginTransformTrait>>,
}

struct OutputPipe {
    //handle: Option<JoinHandle<()>>,
    //id: ThreadId,
    channel_in: Receiver<Arc<RgbData>>,
    name: String,
    output: Box<PluginOutputTrait>,
    transformations: Vec<Box<PluginTransformTrait>>,
}


// main entry point
fn main() {
    // get command line parameter, print usage if config parameter is missing
    let matches = App::new("lightoros")
        .version(crate_version!())
        .about("Flexible LED controlling engine")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // get config parameter and read the file content
    let cfg_path = matches.value_of("config").unwrap();
    let config_file_path = Path::new(cfg_path);
    let content = match fs::read_to_string(&config_file_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Error reading config file '{}': {}", cfg_path, error);
            std::process::exit(1);
        }
    };

    // parse the config file as JSON
    let config: Config = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid format of config file '{}': {}", cfg_path, error);
            std::process::exit(1);
        }
    };
    println!("Config file '{}' loaded.", cfg_path);

    // output plugin channels, main thread will use it to notify output threads about new data
    let mut output_tx = Vec::new();

    // container for the lib references, necessary to prevent the loaded libraries from being dropped
    let mut libs = Vec::new();
    let mut input_threads: Vec<InputThread> = Vec::new();
    let mut output_threads: Vec<OutputThread> = Vec::new();
    let mut input_pipes: Vec<InputPipe> = Vec::new();
    let mut output_pipes: Vec<OutputPipe> = Vec::new();

    // create channels for communications with the threads/plugins
    // input plugin channel. listen on rx for incoming messages from input plugins. tx clones are passed to plugin threads to send data
    // TODO use sync_channel?
    let (tx_input, rx_input) = mpsc::channel();

    // iterate over input plugins
    for input_plugin in &config.plugins.input {
        // create an input pipe which contains one input plugin and optional several transformation plugins
        let input_pipe = match create_input_pipe(tx_input.clone(), input_plugin, &mut libs) {
            Some(pipe) => pipe,
            None => {
                continue;
            }
        };
        input_pipes.push(input_pipe);
    }

    // run each input pipe in a separate thread
    for pipe in input_pipes {
        let name = pipe.name.clone();
        let priority = pipe.priority;
        let thread = thread::Builder::new()
            .name(name)
            .spawn(move || {
                let mut input = pipe.input;
                let tx = pipe.channel_out;
                loop {
                    // get data from input 
                    let mut rgb_data = match input.get() {
                        Some(data) => data,
                        None => {
                            thread::sleep(Duration::from_millis(5000));
                            continue;
                        }
                    };
                    // transform data if necessary
                    for transformator in &pipe.transformations {
                        rgb_data = transformator.transform(&rgb_data);
                    }
                    // send data to the main thread
                    let event = InputEvent::new(Arc::new(rgb_data));
                    tx.send(event).unwrap();
                }
            })
            .unwrap();

        let input_thread = InputThread::new(priority, thread);
        input_threads.push(input_thread);
    }

    // iterate over output plugins
    for output_plugin in &config.plugins.output {
        let (tx_output, rx_output) = mpsc::channel();
        output_tx.push(Box::new(tx_output));

        // create an output pipe which contains one output plugin and optional several transformation plugins
        let output_pipe = match create_output_pipe(rx_output, output_plugin, &mut libs) {
            Some(pipe) => pipe,
            None => {
                continue;
            }
        };
        output_pipes.push(output_pipe);
    }

    // run each output pipe in a separate thread
    for pipe in output_pipes {
        let name = pipe.name.clone();
        let thread = thread::Builder::new()
            .name(name)
            .spawn(move || {
                let output = pipe.output;
                let rx = pipe.channel_in;
                loop {
                    // wait for incoming data
                    let data: Arc<RgbData> = match rx.recv() {
                        Ok(data) => data,
                        Err(error) => {
                            eprintln!("Error receiving data: {}", error);
                            thread::sleep(Duration::from_millis(5000));
                            continue;
                        }
                    };

                    let mut rgb_data_ref: &RgbData = &data;
                    let mut rgb_data: RgbData;
                    // transform data if necessary
                    for transformator in &pipe.transformations {
                        rgb_data = transformator.transform(rgb_data_ref);
                        rgb_data_ref = &rgb_data;
                    }

                    let result = output.send(rgb_data_ref);
                    if !result {
                        eprintln!("Error sending data to output plugin: {}", thread::current().name().unwrap());
                        thread::sleep(Duration::from_millis(5000));
                    }

                }
            })
            .unwrap();

        let output_thread = OutputThread::new(thread);
        output_threads.push(output_thread);
    }

    // main loop, blocks on the channel until data from one of the input plugins arrive and forward it over the channels to the output plugins
    let mut current_priority = 0;
    let mut last_event = Instant::now();
    let inactivity_timeout = config.max_input_inactivity_period;
    loop {
        let event: InputEvent = match rx_input.recv() {
            Ok(event) => {
                // got event from one of the input plugins
                let id: ThreadId = event.thread_id;
                let event_thread = get_thread_by_id(id, &input_threads)
                    .expect("ERROR: Got an event from a not existing thread");

                if current_priority == 0 {
                    // use the current plugin's priority as current
                    current_priority = event_thread.priority;
                } else {
                    // event is coming from a plugin with same or higher priority
                    if event_thread.priority >= current_priority {
                        // the plugin which sent current event has a higher or same priority. Use it as current
                        current_priority = event_thread.priority;
                        last_event = Instant::now();
                    } else {
                        // the plugin which sent current event has a lower priority
                        if last_event.elapsed() >= Duration::from_millis(inactivity_timeout) {
                            current_priority = event_thread.priority;
                            last_event = Instant::now();
                        } else {
                            continue;
                        }
                    }
                }
                event
            }
            Err(error) => {
                eprintln!("Error getting data from input plugins: {}", error);
                continue;
            }
        };

        for tx_out in &output_tx {
            tx_out.send(Arc::clone(&event.data)).unwrap();
        }
    }
}

fn get_thread_by_id(id: ThreadId, input_threads: &Vec<InputThread>) -> Option<&InputThread> {
    input_threads.iter().find(|ref thread| thread.id == id)
}

fn find_plugin_file(name: &str) -> Option<PathBuf> {
    // find plugins
    let paths = fs::read_dir("./target/debug").unwrap();
    for path in paths {
        let full_path = path.unwrap();
        let file_name = full_path.file_name();

        let is_dylib = file_name.to_str().unwrap().contains(".dylib");

        // check input plugins
        let is_plugin = file_name.to_str().unwrap().contains("lightoros_input");
        if is_plugin && is_dylib {
            let mut libfile_path = current_dir().unwrap();
            libfile_path.push(full_path.path());

            let lib = Library::new(libfile_path.as_path()).unwrap();
            let get_info: Symbol<fn() -> PluginInfo> = unsafe { lib.get(b"info").unwrap() };
            let info = get_info();

            if info.name == name {
                return Some(libfile_path);
            }
        }

        // check output plugins
        let is_plugin = file_name.to_str().unwrap().contains("lightoros_output");
        if is_plugin && is_dylib {
            let mut libfile_path = current_dir().unwrap();
            libfile_path.push(full_path.path());

            let lib = Library::new(libfile_path.as_path()).unwrap();
            let get_info: Symbol<fn() -> PluginInfo> = unsafe { lib.get(b"info").unwrap() };
            let info = get_info();

            if info.name == name {
                return Some(libfile_path);
            }
        }
    }
    None
}

fn create_output_plugin(
    name: &str,
    config: &serde_json::Value,
    lib_list: &mut Vec<Library>,
) -> Box<PluginOutputTrait> {
    let path = find_plugin_file(name);
    let lib = Library::new(path.unwrap()).unwrap();
    lib_list.push(lib);
    let lib = lib_list.last().unwrap();
    let create: Symbol<fn(config: &Value) -> Box<PluginOutputTrait>> =
        unsafe { lib.get(b"create").unwrap() };
    create(config)
}

fn create_input_plugin(
    name: &str,
    config: &serde_json::Value,
    lib_list: &mut Vec<Library>,
) -> Box<PluginInputTrait> {
    let path = find_plugin_file(name);
    let lib = Library::new(path.unwrap()).unwrap();
    lib_list.push(lib);
    let lib = lib_list.last().unwrap();
    let create: Symbol<fn(config: &Value) -> Box<PluginInputTrait>> =
        unsafe { lib.get(b"create").unwrap() };
    create(config)
}

fn create_input_pipe(
    channel: Sender<InputEvent>,
    config_value: &serde_json::Value,
    lib_list: &mut Vec<Library>,
) -> Option<InputPipe> {
    if config_value.is_object() {
        // single plugin
        let plugin_info = config_value.as_object().unwrap();
        let plugin_name = match plugin_info.get("name") {
            Some(name) => name.as_str().unwrap(),
            None => {
                eprintln!("Error reading the name of an input plugin");
                return None;
            }
        };
        let ref plugin_config = plugin_info["config"];
        let priority: u8 = match plugin_info.get("priority") {
            Some(priority) => priority.as_u64().unwrap() as u8,
            None => {
                eprintln!("Priority is missing for the input plugin {}", plugin_name);
                0
            }
        };
        let plugin = create_input_plugin(plugin_name, plugin_config, lib_list);
        let mut pipe = InputPipe::new(plugin, channel);
        pipe.priority = priority;
        pipe.name = plugin_name.to_string();
        return Some(pipe);
    } else if config_value.is_array() {
        // plugin chain
        return None;
    }

    None
}

fn create_output_pipe(
    channel: Receiver<Arc<RgbData>>,
    config_value: &serde_json::Value,
    lib_list: &mut Vec<Library>,
) -> Option<OutputPipe> {
    if config_value.is_object() {
        // single plugin
        let plugin_info = config_value.as_object().unwrap();
        let plugin_name = match plugin_info.get("name") {
            Some(name) => name.as_str().unwrap(),
            None => {
                eprintln!("Error reading the name of an input plugin");
                return None;
            }
        };
        let ref plugin_config = plugin_info["config"];

        let plugin = create_output_plugin(plugin_name, plugin_config, lib_list);
        let mut pipe = OutputPipe::new(plugin, channel);
        pipe.name = plugin_name.to_string();
        return Some(pipe);
    } else if config_value.is_array() {
        // plugin chain
        return None;
    }

    None
}

impl InputThread {
    pub fn new(priority: u8, handle: JoinHandle<()>) -> InputThread {
        let id = handle.thread().id();
        InputThread {
            priority,
            handle,
            id,
        }
    }
}

impl OutputThread {
    pub fn new(handle: JoinHandle<()>) -> OutputThread {
        let id = handle.thread().id();
        OutputThread { handle, id }
    }
}



impl InputPipe {
    fn new(input: Box<PluginInputTrait>, channel: Sender<InputEvent>) -> InputPipe {
        InputPipe {
            priority: 0,
            //handle: None,
            name: String::new(),
            input: input,
            channel_out: channel,
            transformations: Vec::new(),
        }
    }
}

impl OutputPipe {
    fn new(output: Box<PluginOutputTrait>, channel: Receiver<Arc<RgbData>>) -> OutputPipe {
        OutputPipe {
            //handle: None,
            name: String::new(),
            output: output,
            channel_in: channel,
            transformations: Vec::new(),
        }
    }
}

impl fmt::Display for InputThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[ Name: {}, ID: {:?} ]",
            self.handle.thread().name().unwrap_or(""),
            self.id
        )
    }
}

impl fmt::Display for OutputThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[ Name: {}, ID: {:?} ]",
            self.handle.thread().name().unwrap_or(""),
            self.id
        )
    }
}