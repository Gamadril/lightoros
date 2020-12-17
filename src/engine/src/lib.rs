use std::thread::ThreadId;

use libloading::{Library, Symbol};
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lightoros_plugin_base::*;

use self::data_types::*;
#[macro_use]
mod data_types;

use log::*;

pub struct LightorosEngine {
    max_input_inactivity_period: u64,
    input_threads: Vec<InputThread>,
    output_threads: Vec<OutputThread>,
    // container for the lib references, necessary to prevent the loaded libraries/plugins from being dropped
    libs: Vec<Library>,
    input_sender: mpsc::Sender<InputEvent>,
    input_receiver: mpsc::Receiver<InputEvent>,
    // output plugin channels, main thread will use it to notify output threads about new data
    output_tx: Vec<Box<Sender<Arc<lightoros_plugin_base::TraitData>>>>,
}

impl LightorosEngine {
    pub fn new() -> LightorosEngine {
        // create a new channel for communications with the threads/pipes
        // input pipe channel: listen on rx for incoming messages from input pipes.
        // tx clones are passed to pipe threads which are using them to send data to the receiver
        // TODO use sync_channel?
        let (sender, receiver) = mpsc::channel();

        android_logger::init_once(android_logger::Config::default().with_min_level(Level::Debug));

        LightorosEngine {
            max_input_inactivity_period: 5000,
            input_threads: Vec::new(),
            output_threads: Vec::new(),
            libs: Vec::new(),
            input_sender: sender,
            input_receiver: receiver,
            output_tx: Vec::new(),
        }
    }
}

pub trait LightorosEngineTrait {
    fn init(&mut self, config_str: String, plugins_path: String) -> Result<(), PluginError>;
    fn start(&mut self);
}

impl LightorosEngineTrait for LightorosEngine {
    // init pipes based on config
    fn init(&mut self, config_str: String, plugins_path: String) -> Result<(), PluginError> {
        // parse the config file as JSON
        let config: Config = match serde_json::from_str(config_str.as_str()) {
            Ok(config) => config,
            Err(error) => return Err(format!("Invalid configuration: {}", error)).unwrap(),
        };
        self.max_input_inactivity_period = config.max_input_inactivity_period;

        // iterate over input pipes
        for input_pipe_description in config.input {
            // create an input pipe which contains one input plugin and optional several transformation plugins
            let input_pipe = create_input_pipe(
                self.input_sender.clone(),
                input_pipe_description,
                &mut self.libs,
                &plugins_path,
            )?;
            //self.input_pipes.push(input_pipe);
            let priority = input_pipe.priority;

            let thread = std::thread::Builder::new()
                .name(input_pipe.name.clone())
                .spawn(move || {
                    let mut input = input_pipe.input;

                    debug!("INIT input: {}", input);
                    let initialized = input.init();
                    debug!("INIT input result: {:?}", initialized);
                    if initialized.is_err() {
                        eprintln!(
                            "[{}] Failed to initialize input plugin '{}': {}",
                            input_pipe.name,
                            input,
                            initialized.err().unwrap()
                        );
                        return;
                    }

                    debug!("LOOP START");
                    loop {
                        // get data from input
                        debug!("WAITNG FOR DATA");
                        let mut data_in = match input.get() {
                            Ok(data) => data,
                            Err(err) => {
                                debug!(
                                    "[{}] Failed getting data from '{}'. sleeping",
                                    input_pipe.name, err
                                );
                                eprintln!(
                                    "[{}] Failed getting data from '{}'. sleeping",
                                    input_pipe.name, err
                                );
                                std::thread::sleep(Duration::from_millis(5000));
                                continue;
                            }
                        };
                        debug!("GOT DATA: {} bytes", data_in.rgb.len());
                        // transform data if necessary
                        for transformator in &input_pipe.transformations {
                            data_in = match transformator.transform(&data_in) {
                                Ok(data) => data,
                                Err(err) => {
                                    eprintln!(
                                        "[{}] Failed transforming data in '{}': {}. sleeping",
                                        input_pipe.name, transformator, err
                                    );
                                    std::thread::sleep(Duration::from_millis(5000));
                                    continue;
                                }
                            };
                        }
                        // send data to the main thread
                        let event = InputEvent::new(Arc::new(data_in));
                        input_pipe.channel_out.send(event).unwrap();
                    }
                })
                .unwrap();

            let input_thread = InputThread::new(priority, thread);
            self.input_threads.push(input_thread);
        }

        // iterate over output pipes
        for output_pipe_description in config.output {
            let (tx_output, rx_output) = mpsc::channel();
            self.output_tx.push(Box::new(tx_output));

            // create an output pipe which contains one output plugin and optional several transformation plugins
            let output_pipe = create_output_pipe(
                rx_output,
                output_pipe_description,
                &mut self.libs,
                &plugins_path,
            )?;
            //self.output_pipes.push(output_pipe);

            let thread = std::thread::Builder::new()
                .name(output_pipe.name.clone())
                .spawn(move || {
                    let mut output = output_pipe.output;
                    let rx = output_pipe.channel_in;

                    debug!("INIT output: {}", output);
                    let initialized = output.init();
                    debug!("INIT output result: {:?}", initialized);
                    if initialized.is_err() {
                        eprintln!(
                            "[{}] Failed to initialize output plugin '{}': {}",
                            output_pipe.name,
                            output,
                            initialized.err().unwrap()
                        );
                        return;
                    }

                    loop {
                        // wait for incoming data
                        let data_in: Arc<TraitData> = match rx.recv() {
                            Ok(data) => data,
                            Err(error) => {
                                debug!("[{}] Error receiving data: {}", output_pipe.name, error);
                                eprintln!("[{}] Error receiving data: {}", output_pipe.name, error);
                                std::thread::sleep(Duration::from_millis(5000));
                                continue;
                            }
                        };
                        debug!("OUTPUT PIPE GOT DATA: {} bytes", data_in.rgb.len());

                        let mut data_ref: &TraitData = &data_in;
                        let mut data_out: TraitData;
                        // transform data if necessary
                        for transformator in &output_pipe.transformations {
                            data_out = match transformator.transform(data_ref) {
                                Ok(data) => data,
                                Err(err) => {
                                    eprintln!(
                                        "[{}] Failed transforming data in '{}': {}. sleeping",
                                        output_pipe.name, transformator, err
                                    );
                                    std::thread::sleep(Duration::from_millis(5000));
                                    continue;
                                }
                            };
                            data_ref = &data_out;
                        }

                        let result = output.send(data_ref);
                        if result.is_err() {
                            eprintln!(
                                "[{}] Failes sending data to '{}': {}. sleeping",
                                output_pipe.name,
                                output,
                                result.err().unwrap()
                            );
                            std::thread::sleep(Duration::from_millis(5000));
                        }
                    }
                })
                .unwrap();

            let output_thread = OutputThread::new(thread);
            self.output_threads.push(output_thread);
        }

        Ok(())
    }

    fn start(&mut self) {
        // main loop, blocks on the channel until data from one of the input pipes arrive and forward it over the channels to the output pipes
        let mut current_priority = 0;
        let mut last_event = Instant::now();
        let inactivity_timeout = self.max_input_inactivity_period;
        loop {
            let event: InputEvent = match self.input_receiver.recv() {
                Ok(event) => {
                    // got event from one of the input plugins
                    let id: ThreadId = event.thread_id;
                    let event_thread = get_thread_by_id(id, &self.input_threads)
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
                    eprintln!("Error getting data from input pipe: {}", error);
                    continue;
                }
            };

            for tx_out in &self.output_tx {
                tx_out.send(Arc::clone(&event.data)).unwrap();
            }
        }
    }
}

fn get_thread_by_id(id: ThreadId, input_threads: &Vec<InputThread>) -> Option<&InputThread> {
    input_threads.iter().find(|ref thread| thread.id == id)
}

fn find_plugin_file(name: &str, folder: &str) -> Result<PathBuf, PluginError> {
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
            let get_info: Symbol<fn() -> PluginInfo> = unsafe { lib.get(b"info").unwrap() };
            let info = get_info();

            if info.name == name {
                return Ok(libfile_path);
            }
        }
    }
    plugin_err!("Cannot find plugin '{}' in folder '{}'", name, folder)
}

fn get_plugin(name: &str, plugins_folder: &str) -> Result<Library, PluginError> {
    let path = find_plugin_file(name, plugins_folder)?;
    Ok(Library::new(path).unwrap())
}

fn create_input_pipe(
    channel: Sender<InputEvent>,
    pipe_description: InputPipeDescription,
    lib_list: &mut Vec<Library>,
    plugins_folder: &str,
) -> Result<InputPipe, PluginError> {
    let input_plugin_info = match pipe_description.members.first() {
        Some(member) => member,
        None => return plugin_err!("Pipe {} is empty.", pipe_description.name),
    };

    let input_plugin_library = get_plugin(&input_plugin_info.kind, plugins_folder)?;
    let get_info: Symbol<fn() -> PluginInfo> =
        unsafe { input_plugin_library.get(b"info").unwrap() };
    let info = get_info();

    if info.kind != PluginKind::Input {
        return plugin_err!(
            "First member of a pipe ({}) must be an input plugin.",
            pipe_description.name
        );
    }

    lib_list.push(input_plugin_library); // prevent deallocation
    let lib = lib_list.last().unwrap();
    let input_plugin = match create_input_plugin!(lib, &input_plugin_info.config) {
        Ok(plugin) => plugin,
        Err(err) => {
            return plugin_err!(
                "Error creating input plugin '{}': {}",
                input_plugin_info.kind,
                err
            );
        }
    };

    let mut pipe = InputPipe::new(input_plugin, channel);
    pipe.priority = pipe_description.priority;
    pipe.name = pipe_description.name.clone();

    if pipe_description.members.len() > 1 {
        for i in 1..pipe_description.members.len() {
            let transform_plugin_info = &pipe_description.members[i];
            let transform_plugin_library = get_plugin(&transform_plugin_info.kind, plugins_folder)?;
            lib_list.push(transform_plugin_library); // prevent deallocation
            let lib = lib_list.last().unwrap();
            let transform_plugin =
                match create_transform_plugin!(lib, &transform_plugin_info.config) {
                    Ok(plugin) => plugin,
                    Err(err) => {
                        return plugin_err!(
                            "Error creating transform plugin '{}': {}",
                            transform_plugin_info.kind,
                            err
                        )
                    }
                };
            pipe.transformations.push(transform_plugin);
        }
    }

    Ok(pipe)
}

fn create_output_pipe(
    channel: Receiver<Arc<TraitData>>,
    pipe_description: OutputPipeDescription,
    lib_list: &mut Vec<Library>,
    plugins_folder: &str,
) -> Result<OutputPipe, PluginError> {
    let output_plugin_info = match pipe_description.members.last() {
        Some(member) => member,
        None => return plugin_err!("Pipe {} is empty.", pipe_description.name),
    };

    let output_plugin_library = get_plugin(&output_plugin_info.kind, plugins_folder)?;
    let get_info: Symbol<fn() -> PluginInfo> =
        unsafe { output_plugin_library.get(b"info").unwrap() };
    let info = get_info();

    if info.kind != PluginKind::Output {
        return plugin_err!(
            "Last member of a pipe ({}) must be an output plugin.",
            pipe_description.name
        );
    }

    lib_list.push(output_plugin_library); // prevent deallocation
    let lib = lib_list.last().unwrap();
    let output_plugin = match create_output_plugin!(lib, &output_plugin_info.config) {
        Ok(plugin) => plugin,
        Err(err) => {
            return plugin_err!(
                "Error creating output plugin '{}': {}",
                output_plugin_info.kind,
                err
            );
        }
    };

    let mut pipe = OutputPipe::new(output_plugin, channel);
    pipe.name = pipe_description.name.clone();

    if pipe_description.members.len() > 1 {
        for i in 0..pipe_description.members.len() - 1 {
            let transform_plugin_info = &pipe_description.members[i];
            let transform_plugin_library = get_plugin(&transform_plugin_info.kind, plugins_folder)?;
            lib_list.push(transform_plugin_library); // prevent deallocation
            let lib = lib_list.last().unwrap();
            let transform_plugin =
                match create_transform_plugin!(lib, &transform_plugin_info.config) {
                    Ok(plugin) => plugin,
                    Err(err) => {
                        return plugin_err!(
                            "Error creating transform plugin '{}': {}",
                            transform_plugin_info.kind,
                            err
                        );
                    }
                };
            pipe.transformations.push(transform_plugin);
        }
    }

    Ok(pipe)
}
