use clap::Args;
use crate::utils::file::{ConfigFile, parse_path_string, write_config_file};

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(long, default_value = ".", help = "Which directory to use as entry, defaults to the current directory")]
    entry: String,
    #[arg(help = "then name of the config file, defaults to the directory name", default_value = "")]
    name: String,
}

pub fn execute(arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, name } = arguments;

    let mut path = parse_path_string(entry)?;
    if path.is_dir() {
        path.push("rask.yaml")
    }

    if path.exists() {
        return Err(format!("Rask already initialised at {:?}", path));
    }

    let mut config_name = name.clone(); // Use clone to avoid modifying the original input
    if config_name.is_empty() {
        config_name = path.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
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