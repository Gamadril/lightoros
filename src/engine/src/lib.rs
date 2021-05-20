use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use lightoros_plugin_base::*;

use data_types::*;
use input_pipe::*;
use output_pipe::*;

#[macro_use]
mod data_types;
mod input_pipe;
mod output_pipe;
mod utils;

pub struct LightorosEngine {
    //max_input_inactivity_period: u64,
    //input_threads: Vec<InputThread>,
    //output_threads: Vec<OutputThread>,
    // container for the lib references, necessary to prevent the loaded libraries/plugins from being dropped
    //libs: Vec<Library>,
    //input_pipe_sender: mpsc::Sender<InputEvent>,
    //input_pipe_sender: Option<mpsc::Sender<InputEvent>>,
    //input_pipe_receiver: mpsc::Receiver<InputEvent>,
    // output plugin channels, main thread will use it to notify output threads about new data
    //output_pipe_sender_list: Vec<Box<Sender<Arc<lightoros_plugin_base::TraitData>>>>,
    input_pipes: Vec<InputPipe>,
    //input_pipe_handles: Vec<JoinHandle<()>>,
    output_pipes: Vec<OutputPipe>,
    //output_pipe_handles: Vec<JoinHandle<()>>,
    //engine: Arc<Mutex<Engine>>,
    engine_sender: Option<Sender<u64>>,
    handle: Option<JoinHandle<()>>,
}

impl LightorosEngine {
    pub fn new() -> LightorosEngine {
        // create a new channel for communications with the threads/pipes
        // input pipe channel: listen on rx for incoming messages from input pipes.
        // tx clones are passed to pipe threads which are using them to send data to the receiver

        LightorosEngine {
            //max_input_inactivity_period: 5000,
            //input_threads: Vec::new(),
            //output_threads: Vec::new(),
            //libs: Vec::new(),
            //input_pipe_sender: input_sender,
            //input_pipe_sender: None,
            //input_pipe_receiver: input_receiver,
            //output_pipe_sender_list: Vec::new(),
            input_pipes: Vec::new(),
            //input_pipe_handles: Vec::new(),
            output_pipes: Vec::new(),
            //output_pipe_handles: Vec::new(),
            //engine: Arc::new(Mutex::new(engine)),
            engine_sender: None,
            handle: None,
        }
    }

    pub fn start(&mut self, config_str: String, plugins_path: String) -> Result<(), PluginError> {
        if self.handle.is_some() {
            return plugin_err!("Cannot start engine, because it's already running.");
        }

        // parse the config file as JSON
        let config: Config = match serde_json::from_str(config_str.as_str()) {
            Ok(config) => config,
            Err(error) => return plugin_err!("Invalid configuration: {}", error),
        };
        let max_input_inactivity_period = config.max_input_inactivity_period;

        let (input_pipe_sender, input_pipe_receiver) = mpsc::channel();
        let mut output_pipe_sender_list = Vec::new();
        //let mut priorities_map: HashMap<ThreadId, u8> = HashMap::new();

        // iterate over input pipes
        for input_pipe_description in config.input {
            // create an input pipe which contains one input plugin and optional several transformation plugins
            let mut input_pipe = InputPipe::create(
                input_pipe_description,
                input_pipe_sender.clone(),
                &plugins_path,
            )?;
            input_pipe.start()?;
            self.input_pipes.push(input_pipe);
        }

        // iterate over output pipes
        for output_pipe_description in config.output {
            let (tx_output, rx_output) = mpsc::channel();
            output_pipe_sender_list.push(Box::new(tx_output));

            // create an output pipe which contains one output plugin and optional several transformation plugins
            let mut output_pipe =
                OutputPipe::create(output_pipe_description, rx_output, &plugins_path)?;
            output_pipe.start()?;
            self.output_pipes.push(output_pipe);
        }

        let (engine_sender, engine_receiver) = mpsc::channel();

        self.engine_sender = Some(engine_sender);

        let handle = std::thread::spawn(move || {
            let mut current_priority = 0;
            let mut last_event = Instant::now();

            // to block, or not to block, that is the question ...
            loop {
                match engine_receiver.try_recv() {
                    Ok(time_to_wait) => {
                        // got signal to exit
                        
                        // send empty data to output pipe as stop signal
                        for tx_out in output_pipe_sender_list.iter() {
                            let data: TraitData = TraitData {
                                rgb: Vec::with_capacity(0),
                                meta: HashMap::new(),
                            }; 
                            tx_out.send(Arc::new(data)).unwrap();
                        }

                        // prevent reading on closed channel after input pipes are closed
                        std::thread::sleep(Duration::from_millis(time_to_wait));
                        return;
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("Error getting command from cmd channel. Disconnected.");
                        return;
                    }
                }

                //let event: InputEvent = match input_pipe_receiver.recv() {
                let event: InputEvent = match input_pipe_receiver.try_recv() {
                    Ok(event) => {
                        // got event from one of the input plugins
                        let priority = event.priority;
                        if current_priority == 0 {
                            // use the current plugin's priority as current
                            current_priority = priority;
                        } else {
                            // event is coming from a plugin with same or higher priority
                            if priority >= current_priority {
                                // the plugin which sent current event has a higher or same priority. Use it as current
                                current_priority = priority;
                                last_event = Instant::now();
                            } else {
                                // the plugin which sent current event has a lower priority
                                if last_event.elapsed()
                                    >= Duration::from_millis(max_input_inactivity_period)
                                {
                                    current_priority = priority;
                                    last_event = Instant::now();
                                } else {
                                    continue;
                                }
                            }
                        }
                        event
                    }
                    Err(TryRecvError::Empty) => continue,
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("Error getting data from input pipe. Disconnected.");
                        return;
                    } /*
                      Err(error) => {
                          eprintln!("Error getting data from input pipe: {}", error);
                          continue;
                          }
                      };
                      */
                };

                for tx_out in output_pipe_sender_list.iter() {
                    tx_out.send(Arc::clone(&event.data)).unwrap();
                }
            }
        });

        self.handle = Some(handle);

        println!("Main engine started");

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), PluginError> {
        if self.handle.is_none() {
            return plugin_err!("Cannot stop engine, because it's not running.");
        }
        println!("Stopping lightoros engine...");

        // stop input pipes
        self.engine_sender.take().unwrap().send(1000).unwrap();
        while let Some(mut pipe) = self.input_pipes.pop() {
            println!("Stopping input pipe {}", pipe);
            pipe.stop()?;
            println!("Stopped {}", pipe);
        }

        // stop output pipes
        while let Some(mut pipe) = self.output_pipes.pop() {
            println!("Stopping output pipe {}", pipe);
            pipe.stop()?;
            println!("Stopped {}", pipe);
        }

        // stop engine
        println!("Stopping main engine");
        self.handle
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");
        println!("Main engine stopped");

        self.handle = None;
        self.engine_sender = None;

        Ok(())
    }
}
