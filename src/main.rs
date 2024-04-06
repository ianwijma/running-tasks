use std::process::exit;
use clap::{Parser, Subcommand};
use commands::run;
use commands::list;
use commands::init;

mod commands;
mod utils;

#[derive(Subcommand, Debug)]
enum Command {
    /// Run specific tasks
    Run(run::Arguments),
    /// List available tasks
    List(list::Arguments),
    /// Initialize a new rask entry
    Init(init::Arguments),
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Rask - The universal way of running tasks", long_about = None, propagate_version = true)]
struct Arguments {
    #[command(subcommand)]
    command: Command
}

fn main() {
    let Arguments { command } = Arguments::parse();

    let result = match command {
        Command::Run(arguments) => { run::execute(&arguments) },
        Command::List(arguments) => { list::execute(&arguments) },
        Command::Init(arguments) => { init::execute(&arguments) },
    };

    match result {
        Ok(_) => exit(0),
        Err(err) => eprintln!("{}", err)
    }
}
