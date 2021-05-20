use clap::crate_version;
use clap::{App, Arg};
use lightoros_engine::*;
use std::fs;
use std::path::Path;

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
        .arg(
            Arg::with_name("plugins dir")
                .short("p")
                .long("plugins_dir")
                .value_name("FOLDER")
                .help("Sets the path to the plugins folder")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    // get config parameter and read the file content
    let cfg_path = matches.value_of("config").unwrap();
    let config_file_path = Path::new(cfg_path);
    let config_str = match fs::read_to_string(&config_file_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Error reading config file '{}': {}", cfg_path, error);
            std::process::exit(1);
        }
    };

    println!("Config file '{}' loaded.", cfg_path);

    let mut exepath = std::env::current_exe().unwrap();
    exepath.pop();
    let plugin_folder = exepath.into_os_string().into_string().unwrap().clone();

    let mut engine = LightorosEngine::new();
    engine.start(config_str.clone(), plugin_folder.clone()).unwrap();

    println!("Will loop");
    loop {
        let mut guess = String::new();

        std::io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let input = guess.trim();
        println!("CMD: {}", input);

        if input == "stop" {
            println!("Stopping engine");
            engine.stop().unwrap()
            
        } else if input == "start" {
            println!("Starting engine");
            engine.start(config_str.clone(), plugin_folder.clone()).unwrap()
        }
    }
}
