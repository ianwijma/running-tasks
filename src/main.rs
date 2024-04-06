use std::process::exit;
use clap::{Parser, Subcommand};
use commands::run;
use commands::list;
use commands::init;

mod commands;
mod utils;

#[derive(Subcommand, Debug)]
enum Command {
    Run(run::Arguments),
    List(list::Arguments),
    Init(init::Arguments),
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
        Command::Run(arguments) => { run::execute(arguments) },
        Command::List(arguments) => { list::execute(arguments) },
        Command::Init(arguments) => { init::execute(arguments) },
    };

    match result {
        Ok(_) => exit(0),
        Err(err) => eprintln!("{}", err)
    }
}
