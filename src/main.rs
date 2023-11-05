use clap::{Args, Parser, Subcommand};
use shit::{create_and_write_file, create_directory};
use std::env;
use std::fs;

#[derive(Debug, Parser)]
pub struct UserInput {
    #[clap(subcommand)]
    init: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// A command description
    Init(Init),
}

#[derive(Debug, Args)]
struct Init {
    /// Some particular parameter command A
    #[clap(short, long, value_parser)]
    name: String,
}

fn main() -> Result<(), std::io::Error> {
    let user_input: UserInput = UserInput::parse();
    match user_input.init {
        Command::Init(init_command) => {
            let curr_path = env::current_dir();
            let dir_path = &format!("{}/{}", curr_path?.display(), init_command.name);

            if fs::metadata(&dir_path).is_ok() {
                eprintln!("Repository {} already exists", init_command.name);
                return Ok(());
            }

            create_directory(&dir_path)?;

            let git_dir_path = &format!("{}/.git", dir_path);
            create_directory(git_dir_path)?;

            create_and_write_file(&format!("{}/HEAD", git_dir_path), "ref: refs/heads/main")?;

            let config_content = "\
                [core]\n\
                repositoryformatversion = 0\n\
                filemode = true\n\
                bare = false\n\
                logallrefupdates = true\n\
                ignorecase = true\n\
                precomposeunicode = true\n";
            create_and_write_file(&format!("{}/config", git_dir_path), config_content)?;

            create_and_write_file(&format!("{}/description", git_dir_path), "")?;

            create_directory(&format!("{}/objects", git_dir_path))?;

            create_directory(&format!("{}/hooks", git_dir_path))?;

            create_directory(&format!("{}/info", git_dir_path))?;
            create_and_write_file(
                &format!("{}/info/exclude", git_dir_path),
                "# git ls-files --others --exclude-from=.git/info/exclude\n\
                 # Lines that start with '#' are comments.\n\
                 # For a project mostly in C, the following would be a good set of\n\
                 # exclude patterns (uncomment them if you want to use them):\n\
                 # *.[oa]\n\
                 # *~\n",
            )?;

            create_directory(&format!("{}/refs", git_dir_path))?;

            println!("Initialized empty repository: {}", init_command.name);
            return Ok(());
        }
    }
}
