use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

pub fn create_directory(path: &str) -> Result<()> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            eprintln!("Oops, error while creating {}: {}", path, err);
            Err(err)
        }
    }
}

pub fn create_and_write_file(path: &str, content: &str) -> Result<()> {
    match File::create(path) {
        Ok(mut file) => {
            file.write_all(content.as_bytes())?;
            Ok(())
        }
        Err(err) => {
            eprintln!("Oops, error while creating {}: {}", path, err);
            Err(err)
        }
    }
}
