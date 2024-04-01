use std::process::exit;
use clap::{Parser, Subcommand};
use commands::run;

mod commands;
mod utils;

#[derive(Subcommand, Debug)]
enum Command {
    Run(run::Arguments)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Arguments {
    #[command(subcommand)]
    command: Command
}

fn main() {
    let arguments = Arguments::parse();

    let result = match &arguments.command {
        Command::Run(arguments) => { run::run(arguments) }
    };

    match result {
        Ok(_) => exit(0),
        Err(err) => eprintln!("{}", err)
    }
}
