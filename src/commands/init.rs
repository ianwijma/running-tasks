use clap::Args;
use crate::utils::file::{ConfigFile, parse_path_string, write_config_file};

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(long, help = "The entry directory or rask.yaml file")]
    entry: Option<String>,
    #[arg(help = "The rask config name, defaults to the directory name")]
    name: Option<String>,
}

pub fn execute(arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, name } = arguments;

    let mut path = parse_path_string(&entry.clone().unwrap_or(".".to_string()))?;
    if path.is_dir() {
        path.push("rask.yaml")
    }

    if path.exists() {
        return Err(format!("Rask already initialised at {:?}", path));
    }

    let config_name: String;
    if name.is_none() {
        config_name = path.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    } else {
        config_name = name.clone().unwrap();
    }

    let config_file: ConfigFile = ConfigFile {
        name: config_name,
        task_engine: Default::default(),
        directories: vec![],
        tasks: Default::default(),
        __file_path: Default::default(),
        __dir_path: Default::default(),
    };

    write_config_file(path.clone(), config_file)?;

    // NOTE: We could improve the init command by adding a reverse search for a parent rake file

    println!("Rask initialised: {:?}", path);

    Ok(())
}