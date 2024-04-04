use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{canonicalize, read_to_string};
use std::path::{Path, PathBuf};
use clap::Args;
use glob::glob;
use globset::{Glob, GlobSetBuilder};
use serde::Deserialize;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(help = "Which task to run")]
    task_name: String,
    #[arg(long, default_value = ".", help = "Which directory to use as entry, defaults to the current directory")]
    entry: String,
}

pub fn run (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, task_name } = arguments;

    // Resolve the entry path
    let entry_config_path: PathBuf = resolve_config_path(entry)?;

    // Discover all config paths
    let config_paths: Vec<PathBuf> = discover_config_paths(&entry_config_path)?;

    // Parse config file content
    let config_files: Vec<ConfigFile> = read_config_files(config_paths)?;

    // Parse config files
    let configs: Vec<Config> = parse_config_files(config_files)?;

    // Resolve dependencies based on the directory structure
    // (In the future this will be configurable based on a dependency config field)
    let config_structure: ConfigStructure = resolve_config_structure(&entry_config_path, configs)?;

    // Gather the tasks from the config
    let task_structure: TaskStructure = resolve_task_structure(config_structure, task_name);
    eprintln!("task_structure = {:?}", task_structure);

    // Run the commands, one by one
    // > In the future this is configurable on the rask level and maybe on the config file level
    // > Initially it fails the whole command if one task fails, but will also be configurable in the future
    // let task_exit: TaskExit = run_task_structure(task_structure);

    Ok(())
}

#[derive(Debug, Clone)]
struct TaskStructure {

}

fn resolve_task_structure(config_structure: ConfigStructure, task_name: &String) -> TaskStructure {
    todo!()
}

#[derive(Debug, Clone)]
struct ConfigStructure {
    config: Config,
    children: Vec<ConfigStructure>
}

fn resolve_config_structure(entry_config_path: &PathBuf, configs: Vec<Config>) -> Result<ConfigStructure, String> {
    let mut path_map: HashMap<PathBuf, Config> = HashMap::new();

    for config in configs {
        path_map.insert(config.clone().path, config);
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

        // TODO: Maybe abstract?
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

#[derive(Debug, Clone)]
enum TaskType {
    SHELL
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ConfigTask {
    task_type: TaskType,
    content: String
}

type ConfigTasks = HashMap<String, ConfigTask>;
type ConfigDirectories = Vec<String>;

#[derive(Debug, Clone, Default, Deserialize)]
enum TaskEngine {
    #[serde(rename = "composer")]
    COMPOSER,
    #[serde(rename = "npm")]
    NPM,
    #[serde(rename = "yarn")]
    YARN,
    #[serde(rename = "pnpm")]
    PNPM,
    #[serde(rename = "none")]
    #[default]
    NONE
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Config {
    name: String,
    task_engine: TaskEngine,
    tasks: HashMap<String, ConfigTask>,
    path: PathBuf,
    directories: ConfigDirectories,
}

fn parse_config_files(config_files: Vec<ConfigFile>) -> Result<Vec<Config>, String> {
    let mut configs: Vec<Config> = vec![];

    for config_file in config_files {
        let config = parse_config_file(config_file)?;
        configs.push(config);
    }

    Ok(configs)
}

fn parse_config_file(config_file: ConfigFile) -> Result<Config, String> {
    let ConfigFile { name, tasks: config_file_tasks, _file_path: file_path, _dir_path: dir_path, directories, task_engine } = config_file;

    let tasks: ConfigTasks = match task_engine {
        TaskEngine::COMPOSER => parse_composer_tasks(dir_path)?,
        TaskEngine::NPM => parse_node_tasks(dir_path, "npm")?,
        TaskEngine::YARN => parse_node_tasks(dir_path, "yarn")?,
        TaskEngine::PNPM => parse_node_tasks(dir_path, "pnmp")?,
        TaskEngine::NONE => parse_config_tasks(config_file_tasks)?,
    };

    let config: Config = Config { name, tasks, path: file_path, directories, task_engine };

    Ok(config)
}

#[derive(Debug, Clone, Deserialize, Default)]
struct PackageJsonFile {
    #[serde(default)]
    scripts: HashMap<String, String>,
}

fn parse_node_tasks(dir_path: PathBuf, prefix: &str) -> Result<ConfigTasks, String> {
    let mut file_path = dir_path.clone();
    file_path.push("package.json");
    let content = read_file_content(file_path)?;

    let package_json: PackageJsonFile = serde_json::from_str(&content).expect(format!("Failed to package.json from \"{:?}\"", dir_path).as_str());

    let mut config_tasks: ConfigTasks = HashMap::new();
    for key in package_json.scripts.keys() {
        config_tasks.insert(key.clone(), ConfigTask {
            task_type: TaskType::SHELL,
            content: format!("{:?} run {:?}", prefix, key)
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

fn parse_composer_tasks(dir_path: PathBuf) -> Result<ConfigTasks, String> {
    let mut file_path = dir_path.clone();
    file_path.push("composer.json");
    let content = read_file_content(file_path)?;

    let package_json: ComposerJsonFile = serde_json::from_str(&content).expect(format!("Failed to composer.json from \"{:?}\"", dir_path).as_str());

    let mut config_tasks: ConfigTasks = HashMap::new();
    for key in package_json.scripts.keys() {
        config_tasks.insert(key.clone(), ConfigTask {
            task_type: TaskType::SHELL,
            content: format!("composer run {:?}", key)
        });
    }

    Ok(config_tasks)
}

fn parse_config_tasks(tasks: ConfigFileTasks) -> Result<ConfigTasks, String> {
    let mut config_tasks: ConfigTasks = HashMap::new();

    for (key, value) in tasks {
        let config_task: ConfigTask = ConfigTask {
            task_type: TaskType::SHELL,
            content: value
        };

        config_tasks.insert(key, config_task);
    }

    Ok(config_tasks)
}

fn read_config_files(paths: Vec<PathBuf>) -> Result<Vec<ConfigFile>, String> {
    let mut configs_files: Vec<ConfigFile> = vec![];

    for path in paths {
        let config_file = read_config_file(path)?;
        configs_files.push(config_file);
    }

    Ok(configs_files)
}

fn discover_config_paths(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    let mut found_config_paths: Vec<PathBuf> = vec![path.clone()];

    // Read config
    let mut path_stack: Vec<PathBuf> = vec![path.clone()];
    while !path_stack.is_empty() {
        let ConfigFile { directories, _file_path, .. } = read_config_file(path_stack.pop().unwrap())?;

        // Extract directories
        let config_directory = _file_path.parent().ok_or("Failed to get parent directory")?;
        for directory in directories {
            let pattern = get_config_glob_pattern(config_directory, &directory);

            // Find config files based on the pattern in the directories value
            let pattern_string: &str = pattern.to_str().unwrap();
            for pattern_results in glob(pattern_string).map_err(|e| format!("Failed to read glob pattern: {}", e))? {
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

fn read_file_content (path: PathBuf) -> Result<String, String> {
    match read_to_string(path) {
        Ok(content) => Ok(content),
        Err(err) => Err(format!("Failed to read file: {}", err)),
    }
}

type ConfigFileTasks = HashMap<String, String>;

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    name: String,
    #[serde(default)]
    task_engine: TaskEngine,
    #[serde(default)]
    directories: ConfigDirectories,
    #[serde(default)]
    tasks: ConfigFileTasks,
    #[serde(default)]
    _file_path: PathBuf,
    #[serde(default)]
    _dir_path: PathBuf,
}

fn read_config_file(path: PathBuf) -> Result<ConfigFile, String> {
    let content = read_file_content(path.clone())?;

    let mut config_file: ConfigFile = serde_yaml::from_str(&content).expect(format!("Failed to parse YAML from \"{:?}\"", path).as_str());

    config_file._file_path = path.clone();
    config_file._dir_path = path.parent().unwrap().to_path_buf();

    Ok(config_file)
}

fn resolve_config_path<P: AsRef<Path> + Debug + Clone + Copy>(path: P) -> Result<PathBuf, String> {
    let full_path = match canonicalize(path) {
        Ok(full_path) => full_path,
        Err(_) => return Err(format!("Target does not exists: {:?}", path.clone()))
    };

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
        let mut possible_config_file = directory_path.clone();
        possible_config_file.push(filename);
        match possible_config_file.exists() {
            true => return Ok(possible_config_file),
            false => {}
        }
    }

    Err(format!("Unable to find a config file (\"{:?}\") in {:?}", CONFIG_FILENAMES, directory_path))
}