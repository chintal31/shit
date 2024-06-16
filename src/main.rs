mod commands;
mod utils;
use crate::commands::Command;
use clap::Parser;

fn main() -> Result<(), std::io::Error> {
    let user_input: Command = Command::parse();
    match user_input {
        Command::Init(init_command) => commands::init_command(init_command),

        Command::Add(add_command) => commands::add_command(add_command),

        Command::Status => commands::status_command(),
    }
}
