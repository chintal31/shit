pub mod add;
pub mod init;
pub mod status;

use add::Add;
use clap::Parser;
use init::Init;

#[derive(Debug, Parser)]
pub enum Command {
    /// Initialize a git repo
    Init(Init),
    /// Add a file to staging area
    Add(Add),
    /// Status of staging area
    Status,
}
