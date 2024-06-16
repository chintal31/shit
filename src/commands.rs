use crate::utils::{compress_and_store, create_and_write_file, create_directory, update_index};
use clap::{Args, Parser};
use sha1::{Digest, Sha1};
use std::env;
use std::fs;

#[derive(Debug, Parser)]
pub enum Command {
    /// Initialize a git repo
    Init(Init),
    /// Add a file to staging area
    Add(Add),
    /// Status of staging area
    Status,
}

#[derive(Debug, Args)]
pub struct Init {
    /// Name of the directory
    #[clap(short, long, value_parser)]
    name: String,
}

#[derive(Debug, Args)]
pub struct Add {
    /// Name of the file to be added to staging area
    #[clap(short, long, value_parser)]
    file_name: String,
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

pub fn add_command(add_command: Add) -> Result<(), std::io::Error> {
    let curr_path = env::current_dir()?;
    let file_path = &format!("{}/{}", curr_path.display(), add_command.file_name);
    let content = fs::read(file_path)?;
    let mut hasher = Sha1::new();
    let header = format!("blob {}\0", content.len());
    hasher.update(header.as_bytes());
    hasher.update(&content);
    let hash_arr = hasher.finalize();
    let hash = format!("{:x}", hash_arr);
    println!("Hash: {}", hash);
    let content_to_store = format!("{}{:?}", header, &content);
    let zip_path = &format!("{}/.git/objects/{}", curr_path.display(), &hash[..2]);
    println!("Zip path: {}", zip_path);
    create_directory(zip_path)?;
    let zip_file_name = &format!("{}/{}", zip_path, &hash[2..]);
    println!("Zip file name: {}", zip_file_name);
    compress_and_store(content_to_store.as_bytes(), zip_file_name)?;
    update_index(file_path, &hash)?;
    return Ok(());
}

pub fn status_command() -> Result<(), std::io::Error> {
    print!("On branch main\n\n");
    Ok(())
}
