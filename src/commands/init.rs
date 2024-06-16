use crate::utils::{create_and_write_file, create_directory};
use clap::Args;
use std::env;
use std::fs;

#[derive(Debug, Args)]
pub struct Init {
    /// Name of the directory
    #[clap(short, long, value_parser)]
    name: String,
}

pub fn init_command(init_command: Init) -> Result<(), std::io::Error> {
    let curr_path = env::current_dir()?;
    let dir_path = &format!("{}/{}", curr_path.display(), init_command.name);

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
