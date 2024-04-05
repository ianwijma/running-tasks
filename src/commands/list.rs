use clap::Args;

#[derive(Args, Debug)]
pub struct Arguments {
    #[arg(long, default_value = ".", help = "Which directory to use as entry, defaults to the current directory")]
    entry: String,
}

pub fn execute (arguments: &Arguments) -> Result<(), String> {
    let Arguments { entry } = arguments;

    eprintln!("entry = {:?}", entry);

    Ok(())
}