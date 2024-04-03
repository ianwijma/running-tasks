use clap::Args;
use crate::utils::config::{parse_config, validate_config};
use crate::utils::file_resolvers::resolve_configuration_file;
use crate::utils::tasks::run_task;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg()]
    command: String,
    #[arg(long, default_value = ".")]
    entry: String,
}

pub fn run (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry, command } = arguments;

    let target = resolve_configuration_file(entry)?;

    let config = parse_config(&target)?;

    match validate_config(config.clone()) {
        Ok(_) => {}
        Err(err) => return Err(err)
    }

    for config in config.clone().iter() {
        match config.get_task(command) {
            None => {}
            Some(task) => {
                match run_task(&task) {
                    Ok(_) => {}
                    Err(err) => return Err(err)
                }
            }
        }
    }

    println!("{:?}", config);

    Ok(())
}