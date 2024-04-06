use std::path::{Path, PathBuf};
use std::fmt::Debug;
use std::collections::HashMap;
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;
use crate::utils::file;
use crate::utils::file::{ConfigFile, ConfigFileTasks, TaskEngine};

#[derive(Debug, Clone)]
pub enum TaskExit {
    SUCCESS,
    FAILURE
}

#[derive(Debug, Clone)]
pub struct Task {
    pub command: String,
    pub directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OrderedTask {
    pub task: Task,
    pub order: u64
}

pub type OrderedTasks = Vec<OrderedTask>;

pub fn resolve_task_order(config_structure: ConfigStructure, task_name: &String) -> Result<OrderedTasks, String> {
    let mut ordered_tasks: OrderedTasks = vec![];

    order_tasks(&mut ordered_tasks, config_structure, task_name, 0);

    Ok(ordered_tasks)
}

fn order_tasks(ordered_tasks: &mut OrderedTasks, config_structure: ConfigStructure, task_name: &String, index: u64) {
    let ConfigStructure { config, children } = config_structure;
    let Config { tasks, dir_path, .. } = config;

    for config_task in tasks {
        let ConfigTask { ref key, .. } = config_task;
        if key == task_name {
            ordered_tasks.push(OrderedTask {
                task: Task {
                    command: resolve_config_task_command(&config_task.clone()),
                    directory: dir_path.clone(), // compiler says it's being moved, No idea where...
                },
                order: index
            })
        }
    }

    for child in children {
        order_tasks(ordered_tasks, child, task_name, index+1);
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStructure {
    pub config: Config,
    pub children: Vec<ConfigStructure>
}

pub fn resolve_config_structure(entry_config_path: &PathBuf, configs: Vec<Config>) -> Result<ConfigStructure, String> {
    let mut path_map: HashMap<PathBuf, Config> = HashMap::new();

    for config in configs {
        path_map.insert(config.clone().file_path, config);
    }

    let config_structure: ConfigStructure = construct_config_structure(entry_config_path, &path_map)?;

    Ok(config_structure)
}

fn construct_config_structure(config_path: &PathBuf, config_path_map: &HashMap<PathBuf, Config>) -> Result<ConfigStructure, String> {
    let config = config_path_map.get(config_path).ok_or("Unknown config path")?;

    let paths: Vec<PathBuf> = config_path_map.keys().cloned().collect();
    let Config { directories, .. } = config;
    let config_directory: &Path = config_path.parent().unwrap();
    let mut child_paths: Vec<PathBuf> = vec![];

    for directory in directories {
        let path_pattern: PathBuf = get_config_glob_pattern(config_directory, &directory);

        let pattern = match Glob::new(path_pattern.to_str().unwrap()) {
            Ok(pattern) => pattern,
            Err(err) => return Err(format!("Failed to create glob pattern: {:?}", err)),
        };
        let mut builder = GlobSetBuilder::new();
        builder.add(pattern);
        let glob_set = builder.build().unwrap();

        for path in &paths {
            if glob_set.is_match(path) {
                child_paths.push(path.to_path_buf());
            }
        }
    }

    let config_structure = ConfigStructure {
        config: config.clone(),
        children: child_paths
            .iter()
            .map(|path| construct_config_structure(path, config_path_map).unwrap())
            .collect()
    };

    Ok(config_structure)
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TaskType {
    #[default]
    SHELL,
    COMPOSER,
    NPM,
    YARN,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConfigTask {
    pub(crate) task_type: TaskType,
    pub(crate) key: String,
    pub(crate) value: String,
}

pub fn resolve_config_task_command(config_task: &ConfigTask) -> String {
    let ConfigTask { task_type, key, value } = config_task;

    match task_type {
        TaskType::SHELL => value.clone(),
        TaskType::COMPOSER => format!("composer run {}", key),
        TaskType::NPM => format!("npm run {}", key),
        TaskType::YARN => format!("yarn run {}", key),
    }
}

type ConfigTasks = Vec<ConfigTask>;
type ConfigDirectories = Vec<String>;

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) tasks: ConfigTasks,
    pub(crate) file_path: PathBuf,
    pub(crate) dir_path: PathBuf,
    pub(crate) directories: ConfigDirectories,
}

pub fn parse_config_files(config_files: Vec<ConfigFile>) -> Result<Vec<Config>, String> {
    let mut configs: Vec<Config> = vec![];

    for config_file in config_files {
        let config = parse_config_file(config_file)?;
        configs.push(config);
    }

    Ok(configs)
}

fn parse_config_file(config_file: ConfigFile) -> Result<Config, String> {
    let ConfigFile { name, directories, task_engine, tasks: config_file_tasks, .. } = config_file;
    let ConfigFile { __file_path: file_path, __dir_path: dir_path, .. } = config_file;

    let tasks: ConfigTasks = match task_engine {
        TaskEngine::COMPOSER => parse_composer_json_tasks(&dir_path)?,
        TaskEngine::NPM => parse_package_json_tasks(&dir_path, TaskType::NPM)?,
        TaskEngine::YARN => parse_package_json_tasks(&dir_path, TaskType::YARN)?,
        TaskEngine::NONE => parse_config_tasks(config_file_tasks)?,
        TaskEngine::AUTO => parse_discovered_tasks(&dir_path, config_file_tasks)?,
    };

    let config: Config = Config { name, tasks, file_path, dir_path, directories };

    Ok(config)
}

const PACKAGE_JSON_FILE: &str = "package.json";
const YARN_LOCK_FILE: &str = "yarn.lock";
const COMPOSER_JSON_FILE: &str = "composer.json";

fn parse_discovered_tasks(dir_path: &PathBuf, config_file_tasks: ConfigFileTasks) -> Result<ConfigTasks, String> {
    let mut config_tasks: ConfigTasks = parse_config_tasks(config_file_tasks)?;

    // Gathering facts
    let has_composer_json = dir_path.join(COMPOSER_JSON_FILE).exists();
    let has_package_json = dir_path.join(PACKAGE_JSON_FILE).exists();
    let has_yarn_lock = dir_path.join(YARN_LOCK_FILE).exists();


    if has_composer_json {
        let composer_config_tasks = parse_composer_json_tasks(dir_path)?;
        config_tasks.extend(composer_config_tasks.into_iter())
    }

    if has_package_json {
        let package_config_tasks: ConfigTasks;

        if has_yarn_lock {
            package_config_tasks = parse_package_json_tasks(dir_path, TaskType::YARN)?;
        } else {
            // No lock file, for now we assume the uses intends to use NPM.
            package_config_tasks = parse_package_json_tasks(dir_path, TaskType::NPM)?;
        }

        config_tasks.extend(package_config_tasks);
    }

    Ok(config_tasks)
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PackageJsonFile {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

fn parse_package_json_tasks(dir_path: &PathBuf, task_type: TaskType) -> Result<ConfigTasks, String> {
    let package_json = file::read_json_file::<PackageJsonFile>(&dir_path.join(PACKAGE_JSON_FILE))?;

    let mut config_tasks: ConfigTasks = vec![];
    for key in package_json.scripts.keys() {
        config_tasks.push(ConfigTask {
            task_type,
            key: key.clone(),
            value: key.clone()
        });
    }

    Ok(config_tasks)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ComposerJsonScriptValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ComposerJsonFile {
    #[serde(default)]
    scripts: HashMap<String, ComposerJsonScriptValue>,
}

fn parse_composer_json_tasks(dir_path: &PathBuf) -> Result<ConfigTasks, String> {
    let package_json = file::read_json_file::<ComposerJsonFile>(&dir_path.join(COMPOSER_JSON_FILE))?;

    let mut config_tasks: ConfigTasks = vec![];
    for key in package_json.scripts.keys() {
        config_tasks.push(ConfigTask {
            task_type: TaskType::COMPOSER,
            key: key.clone(),
            value: key.clone(),
        });
    }

    Ok(config_tasks)
}

fn parse_config_tasks(tasks: ConfigFileTasks) -> Result<ConfigTasks, String> {
    let mut config_tasks: ConfigTasks = vec![];

    for (key, value) in tasks {
        config_tasks.push(ConfigTask {
            task_type: TaskType::SHELL,
            key,
            value
        });
    }

    Ok(config_tasks)
}

pub fn read_config_files(config_file_paths: Vec<PathBuf>) -> Result<Vec<ConfigFile>, String> {
    let mut configs_files: Vec<ConfigFile> = vec![];

    for config_file_path in config_file_paths {
        let config_file = file::read_config_file(config_file_path)?;
        configs_files.push(config_file);
    }

    Ok(configs_files)
}

pub fn discover_config_paths(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut found_config_paths: Vec<PathBuf> = vec![path.clone()];

    // Read config
    let mut path_stack: Vec<PathBuf> = vec![path.clone()];
    while !path_stack.is_empty() {
        let ConfigFile { directories, __file_path: _file_path, .. } = file::read_config_file(path_stack.pop().unwrap())?;

        // Extract directories
        let config_directory = _file_path.parent().ok_or("Failed to get parent directory")?;
        for directory in directories {
            let pattern = get_config_glob_pattern(config_directory, &directory);

            // Find config files based on the pattern in the directories value
            let pattern_string: &str = pattern.to_str().unwrap();
            for pattern_results in glob::glob(pattern_string).map_err(|e| format!("Failed to read glob pattern: {}", e))? {
                if let Ok(found_config_path) = pattern_results {
                    // Only add if the path was not already processed, preventing loops.
                    if !found_config_paths.contains(&found_config_path) {
                        found_config_paths.push(found_config_path.clone());
                        path_stack.push(found_config_path.clone());
                    }
                }
            }
        }
    }

    Ok(found_config_paths)
}

fn get_config_glob_pattern(root_path: &Path, glob_pattern: &String) -> PathBuf {
    let mut pattern: PathBuf = root_path.to_path_buf();

    pattern.push(glob_pattern);
    if !pattern.ends_with(".yaml") {
        pattern.push("rask.yaml");
    }

    pattern
}

pub fn resolve_config_path<P: AsRef<Path> + Debug + Clone + Copy>(path: P) -> Result<PathBuf, String> {
    let full_path = file::parse_path_string(path)?;

    if full_path.is_dir() {
        let config_file = find_config_file(full_path)?;
        return Ok(config_file)
    }

    Ok(full_path)
}

const CONFIG_FILENAMES: [&str; 1] = ["rask.yaml"];

fn find_config_file(directory_path: PathBuf) -> Result<PathBuf, String> {
    if !directory_path.is_dir() {
        return Err(format!("\"{:?}\" is not a directory", directory_path))
    }

    for filename in CONFIG_FILENAMES {
        let possible_config_file = directory_path.join(filename);
        match possible_config_file.exists() {
            true => return Ok(possible_config_file),
            false => {}
        }
    }

    Err(format!("Unable to find a config file (\"{:?}\") in {:?}", CONFIG_FILENAMES, directory_path))
}
