use std::collections::HashMap;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use glob::glob;

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    name: String,
    #[serde(default)]
    directories: Vec<String>,
    #[serde(default)]
    tasks: HashMap<String, String>,
}

fn read_config_file(path_buf: &PathBuf) -> Result<ConfigFile, String> {
    let file = File::open(path_buf).expect("Failed to read file");

    let reader = BufReader::new(file);

    let config: ConfigFile = serde_yaml::from_reader(reader).expect("Failed to parse YAML");

    Ok(config)
}

#[derive(Debug)]
pub struct ConfigTask {
    tag: String,
    command: String
}

#[derive(Debug)]
pub struct SubConfig {
    config: Config,
    path: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    name: String,
    tasks: Vec<ConfigTask>,
    sub_configs: Vec<SubConfig>,
}

pub fn parse_config(path_buf: &PathBuf) -> Result<Config, String> {
    let config_file = read_config_file(path_buf)?;

    let name = config_file.name;

    let tasks = config_file.tasks
        .iter()
        .map(|(tag, command)| ConfigTask{tag: tag.clone(), command: command.clone()})
        .collect();

    let mut sub_configs: Vec<SubConfig> = Vec::new();

    let parent_dir = path_buf.parent().ok_or("Failed to get parent directory")?;

    for directory in config_file.directories {
        let mut pattern: PathBuf = parent_dir.to_path_buf();
        pattern.push(&directory);
        if !pattern.ends_with(".yaml") {
            pattern.push("rask.yaml");
        }

        for entry in glob(pattern.to_str().unwrap()).map_err(|e| format!("Failed to read glob pattern: {}", e))? {
            if let Ok(config_path) = entry {
                let sub_config = SubConfig {
                    config: parse_config(&config_path)?,
                    path: config_path,
                };
                sub_configs.push(sub_config);
            }
        }
    }

    let config = Config{name, tasks, sub_configs};

    Ok(config)
}