mod commands;
mod utils;
use crate::commands::{add, init, status, Command};
use clap::Parser;

fn main() -> Result<(), std::io::Error> {
    let user_input: Command = Command::parse();
    match user_input {
        Command::Init(init_command) => init::init_command(init_command),

        Command::Add(add_command) => add::add_command(add_command),

        Command::Status => status::status_command(),
    }
}
