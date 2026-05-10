mod cli;
mod commands;
mod output;
mod update;

pub use cli::{channel, help_text, version};

pub fn run(args: Vec<String>) -> Result<(), String> {
    cli::run(args)
}
