mod bgp_state;

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
    file_list: Vec<String>,
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
    // Parse command line arguments
    let args = Args::parse();
    
    // Load configuration from the specified file
    let config = load_config(&args.config)?;
    
    println!("Loaded configuration from: {}", args.config);
    println!("Config: {:?}", config);
    
    Ok(())
}
