use std::path::{Path, PathBuf};
use std::fs::{canonicalize, read_to_string, write};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub fn read_file_content (path: PathBuf) -> Result<String, String> {
    match read_to_string(path) {
        Ok(content) => Ok(content),
        Err(err) => Err(format!("Failed to read file: {}", err)),
    }
}

pub fn write_file_content(file_path: &PathBuf, content: &str) -> Result<(), String> {
    write(file_path, content).map_err(|err| format!("Failed to write to file: {}", err))?;

    Ok(())
}

pub fn read_json_file<T: for<'a> Deserialize<'a>>(file_path: &PathBuf) -> Result<T, String> {
    let content = read_file_content(file_path.clone())?;

    let file_content: T = serde_json::from_str::<T>(&content).map_err(|err| err.to_string())?;

    Ok(file_content)
}

fn read_yaml_file<T: for<'a> Deserialize<'a>>(file_path: &PathBuf) -> Result<T, String> {
    let content = read_file_content(file_path.clone())?;

    let file_content: T = serde_yaml::from_str::<T>(&content).map_err(|err| err.to_string())?;

    Ok(file_content)
}

pub fn write_yaml_file<T: Serialize + Debug>(file_path: &PathBuf, data: &T) -> Result<(), String> {
    let yaml_content = serde_yaml::to_string(data).map_err(|err| err.to_string())?;

    write_file_content(file_path, &yaml_content)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskEngine {
    COMPOSER,
    NPM,
    YARN,
    NONE,
    #[default]
    AUTO,
}

pub type ConfigFileTasks = HashMap<String, String>;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "is_default_task_engine")]
    pub(crate) task_engine: TaskEngine,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) directories: Vec<String>,
    #[serde(default, skip_serializing_if = "ConfigFileTasks::is_empty")]
    pub(crate) tasks: ConfigFileTasks,
    // The following fields are not part of the yaml file.
    #[serde(default, skip_serializing_if = "skip_path")]
    pub(crate) __file_path: PathBuf,
    #[serde(default, skip_serializing_if = "skip_path")]
    pub(crate) __dir_path: PathBuf,
}

fn is_default_task_engine(value: &TaskEngine) -> bool {
    match value {
        // Changing the default is generally discouraged.
        TaskEngine::AUTO => true,
        _ => false,
    }
}

fn skip_path(path: &PathBuf) -> bool {
    path.to_str().map_or(true, |s| s.is_empty())
}

pub fn read_config_file(config_file_path: PathBuf) -> Result<ConfigFile, String> {
    let mut config_file = read_yaml_file::<ConfigFile>(&config_file_path)?;

    config_file.__file_path = config_file_path.clone();
    config_file.__dir_path = config_file_path.parent().unwrap().to_path_buf();

    Ok(config_file)
}

pub fn write_config_file(config_file_path: PathBuf, config_file: ConfigFile) -> Result<(), String> {
    write_yaml_file::<ConfigFile>(&config_file_path, &config_file)
}

pub fn parse_path_string<P: AsRef<Path> + Debug + Clone + Copy>(path: P) -> Result<PathBuf, String> {
    let full_path = match canonicalize(path) {
        Ok(full_path) => full_path,
        Err(_) => return Err(format!("Target does not exists: {:?}", path.clone()))
    };

    Ok(full_path)
}

