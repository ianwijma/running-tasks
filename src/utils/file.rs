use std::path::PathBuf;
use std::fs::read_to_string;
use std::collections::HashMap;
use serde::Deserialize;

pub fn read_file_content (path: PathBuf) -> Result<String, String> {
    match read_to_string(path) {
        Ok(content) => Ok(content),
        Err(err) => Err(format!("Failed to read file: {}", err)),
    }
}

pub fn read_json_file<T: for<'a> Deserialize<'a>>(file_path: &PathBuf) -> Result<T, String> {
    let content = read_file_content(file_path.clone())?;

    let file_content: T = serde_json::from_str::<T>(&content).expect(format!("Failed to read the file: \"{:?}\"", file_path).as_str());

    Ok(file_content)
}

fn read_yaml_file<T: for<'a> Deserialize<'a>>(file_path: &PathBuf) -> Result<T, String> {
    let content = read_file_content(file_path.clone())?;

    let file_content: T = serde_yaml::from_str::<T>(&content).expect(format!("Failed to read the file: \"{:?}\"", file_path).as_str());

    Ok(file_content)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub enum TaskEngine {
    #[serde(rename = "composer")]
    COMPOSER,
    #[serde(rename = "npm")]
    NPM,
    #[serde(rename = "yarn")]
    YARN,
    #[serde(rename = "none")]
    NONE,
    #[serde(rename = "auto")]
    #[default]
    AUTO,
}

pub type ConfigFileTasks = HashMap<String, String>;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ConfigFile {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) task_engine: TaskEngine,
    #[serde(default)]
    pub(crate) directories: Vec<String>,
    #[serde(default)]
    pub(crate) tasks: ConfigFileTasks,
    // The following fields are not part of the yaml file.
    #[serde(default)]
    pub(crate) __file_path: PathBuf,
    #[serde(default)]
    pub(crate) __dir_path: PathBuf,
}

pub fn read_config_file(config_file_path: PathBuf) -> Result<ConfigFile, String> {
    let mut config_file = read_yaml_file::<ConfigFile>(&config_file_path)?;

    config_file.__file_path = config_file_path.clone();
    config_file.__dir_path = config_file_path.parent().unwrap().to_path_buf();

    Ok(config_file)
}

