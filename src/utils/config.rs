use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use std::collections::HashMap;


pub fn validate_config(config: Config) -> Result<bool, String> {
    let mut names: Vec<String> = vec![];

    for config in config.iter() {
        let Config { name, path, .. } = config;

        if names.contains(&name) {
            return Err(format!("Duplicate config name {} found: {:?}", name, path));
        }

        names.push(name);
    }

    Ok(true)
}

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

#[derive(Debug, Clone)]
pub struct ConfigTask {
    tag: String,
    command: String
}

#[derive(Debug, Clone)]
pub struct Config {
    name: String,
    tasks: Vec<ConfigTask>,
    path: PathBuf,
    sub_configs: Vec<Config>,
}

pub struct ConfigIterator {
    stack: Vec<Config>
}

impl ConfigIterator {
    pub fn new(config: Config) -> ConfigIterator {
        let mut stack = vec![config];
        ConfigIterator{ stack }
    }
}

impl Iterator for ConfigIterator {
    type Item = Config;

    fn next(&mut self) -> Option<Self::Item> {
        let next_config = self.stack.pop()?;

        self.stack.extend(next_config.sub_configs.iter().rev().map(|sub_config| sub_config.clone()));

        Some(next_config)
    }
}

impl Config {
    fn iter(self) -> ConfigIterator {
        return ConfigIterator::new(self.clone());
    }
}

pub fn parse_config(path: &PathBuf) -> Result<Config, String> {
    let config_file = read_config_file(path)?;

    let name = config_file.name;

    let tasks = config_file.tasks
        .iter()
        .map(|(tag, command)| ConfigTask{tag: tag.clone(), command: command.clone()})
        .collect();

    let mut sub_configs: Vec<Config> = Vec::new();

    let parent_dir = path.parent().ok_or("Failed to get parent directory")?;

    for directory in config_file.directories {
        let mut pattern: PathBuf = parent_dir.to_path_buf();
        pattern.push(&directory);
        if !pattern.ends_with(".yaml") {
            pattern.push("rask.yaml");
        }

        for entry in glob::glob(pattern.to_str().unwrap()).map_err(|e| format!("Failed to read glob pattern: {}", e))? {
            if let Ok(config_path) = entry {
                let sub_config = parse_config(&config_path)?;
                sub_configs.push(sub_config);
            }
        }
    }

    let config = Config{name, tasks, sub_configs, path: path.clone()};

    Ok(config)
}
