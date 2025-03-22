mod bgp_state;
mod mrt_processor;
mod util;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

// Define a struct that represents your YAML data structure
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    initial_state: Option<String>,
    update_files: Vec<String>,
}

// Function to load config from YAML file
fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration from the specified file
    let config = load_config(&args.config)?;

    log::info!("Loaded configuration from: {}", args.config);
    log::debug!("Config: {:?}", config);

    let mut processor = mrt_processor::MrtProcessor::new(180,Some(3));
    config.initial_state.map(|file| processor.process_bview(file));

    for file in &config.update_files {
        processor.process_update_file(file)?;
    }


    Ok(())
}
