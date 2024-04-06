use std::path::PathBuf;
use clap::Args;
use crate::utils::config;
use crate::utils::config::{Config, ConfigTask};
use crate::utils::file::ConfigFile;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(long, default_value = ".", help = "Which directory to use as entry, defaults to the current directory")]
    entry: String,
}

pub fn execute (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry } = arguments;

    // Resolve the entry path
    let entry_config_path: PathBuf = config::resolve_config_path(entry)?;

    // Discover all config paths
    let config_file_paths: Vec<PathBuf> = config::discover_config_paths(&entry_config_path)?;

    // Parse config file content
    let config_files: Vec<ConfigFile> = config::read_config_files(config_file_paths)?;

    // Parse config files
    let configs: Vec<Config> = config::parse_config_files(config_files)?;

    // get all available tasks
    let tasks: Vec<String> = get_config_tasks(&configs)?;

    println!("The following tasks are available:");
    for task in tasks {
        println!("  -  {}", task)
    }

    Ok(())
}

fn get_config_tasks(configs: &Vec<Config>) -> Result<Vec<String>, String> {
    let mut tasks: Vec<String> = vec![];

    for config in configs {
        for configTask in &config.tasks {
            let ConfigTask { key, .. } = configTask;
            if !tasks.contains(&key) {
                tasks.push(key.clone());
            }
        }
    }

    Ok(tasks)
}