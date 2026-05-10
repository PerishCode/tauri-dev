mod cli;
mod commands;
mod output;

pub use cli::{help_text, version};

pub fn run(args: Vec<String>) -> Result<(), String> {
    cli::run(args)
}
