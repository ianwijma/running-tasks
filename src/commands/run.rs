use clap::Args;
use crate::utils::config_reader::parse_config;
use crate::utils::file_resolvers::resolve_configuration_file;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg()]
    command: String,
    #[arg(default_value = ".")]
    target: String,
}

pub fn run (arguments: &Arguments) -> Result<(), String> {
    let Arguments { target, command } = arguments;

    let target = resolve_configuration_file(target)?;

    let config = parse_config(&target)?;

    println!("{:?}", config);

    Ok(())
}