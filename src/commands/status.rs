use crate::constants::{INDEX_FILE, PREFIX_SIZE};
use crate::utils::IndexEntry;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, Read};
use walkdir::WalkDir;

pub fn status_command() -> Result<(), std::io::Error> {
    print!("On branch main\n\n");

    // loop through the index file and print the file names
    let entries = get_index_entries()?;

    // Traverse directory and compare files
    let (changes_to_be_committed, untracked_files) = traverse_directory_and_compare(".", &entries)?;

    // Log the results
    log(&changes_to_be_committed, &untracked_files);

    Ok(())
}

fn get_index_entries() -> io::Result<Vec<IndexEntry>> {
    let mut file = File::open(INDEX_FILE)?;

    let mut signature = [0; 4];
    file.read_exact(&mut signature)?;
    if &signature != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid index file signature",
        ));
    }

    let _version = file.read_u32::<BigEndian>()?;
    let entry_count = file.read_u32::<BigEndian>()?;

    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        let decoded_index_entry = decode_index_entry(&mut file)?;

        entries.push(decoded_index_entry);
    }

    Ok(entries)
}

pub fn decode_index_entry(file: &mut File) -> Result<IndexEntry, std::io::Error> {
    let ctime_sec = file.read_u32::<BigEndian>()?;
    let ctime_nsec = file.read_u32::<BigEndian>()?;
    let mtime_sec = file.read_u32::<BigEndian>()?;
    let mtime_nsec = file.read_u32::<BigEndian>()?;
    let dev = file.read_u32::<BigEndian>()?;
    let ino = file.read_u32::<BigEndian>()?;
    let mode = file.read_u32::<BigEndian>()?;
    let uid = file.read_u32::<BigEndian>()?;
    let gid = file.read_u32::<BigEndian>()?;
    let size = file.read_u32::<BigEndian>()?;
    let mut hash = vec![0; 20];
    file.read_exact(&mut hash)?;
    let flags = file.read_u16::<BigEndian>()?;
    let path_length = (flags & 0xFFF) as usize;
    let mut path_bytes = vec![];
    for _ in 0..path_length {
        let byte = file.read_u8()?;
        if byte == 0 {
            break;
        }
        path_bytes.push(byte);
    }
    let padding_length = (8 - (PREFIX_SIZE + path_length) % 8) % 8;
    let mut padding = vec![0; padding_length];
    file.read_exact(&mut padding)?;

    Ok(IndexEntry {
        ctime_sec,
        ctime_nsec,
        mtime_sec,
        mtime_nsec,
        dev,
        ino,
        mode,
        uid,
        gid,
        size,
        hash: hex::encode(hash),
        name: String::from_utf8(path_bytes).unwrap(),
        stage: (flags >> 12) as u16,
    })
}

fn traverse_directory_and_compare(
    directory: &str,
    index_entries: &[IndexEntry],
) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
    let mut changes_to_be_committed = vec![];
    let mut untracked_files = vec![];

    // Collect file names from the index entries and sort them
    let mut index_files: Vec<&str> = index_entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    index_files.sort();

    // Traverse the directory
    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            println!("{:?}", file_name);

            //TODO: Skip the .git directory

            // Use binary search to check if the path exists in the sorted list of index files
            match index_files.binary_search(&file_name) {
                Ok(_) => changes_to_be_committed.push(file_name.to_string()), // Path found in the index
                Err(_) => untracked_files.push(file_name.to_string()), // Path not found in the index
            }
        }
    }

    Ok((changes_to_be_committed, untracked_files))
}

fn log(changes_to_be_committed: &[String], untracked_files: &[String]) {
    if !changes_to_be_committed.is_empty() {
        println!("Changes to be committed:");
        for file in changes_to_be_committed {
            println!("\tnew file:   {}", file);
        }
    }
    if !untracked_files.is_empty() {
        println!("\nUntracked files:");
        for file in untracked_files {
            println!("\t{}", file);
        }
    }
}
