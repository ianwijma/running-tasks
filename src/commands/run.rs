use clap::Args;
use crate::utils::config_reader::parse_config;
use crate::utils::file_resolvers::resolve_configuration_file;

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

    println!("{:?}", config);

    Ok(())
}