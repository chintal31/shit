use crate::utils::{compress_and_store, create_directory, update_index};
use clap::Args;
use sha1::{Digest, Sha1};
use std::env;
use std::fs;

#[derive(Debug, Args)]
pub struct Add {
    /// Name of the file to be added to staging area
    #[clap(short, long, value_parser)]
    file_name: String,
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
