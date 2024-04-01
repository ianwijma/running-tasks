use clap::Args;
use crate::utils::file_resolvers::resolve_configuration_file;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(default_value = ".")]
    target: String
}

pub fn run (arguments: &Arguments) -> Result<(), String> {
    let target = resolve_configuration_file(&arguments.target)?;

    println!("{:?}", target);

    Ok(())
}