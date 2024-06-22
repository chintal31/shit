use byteorder::{BigEndian, WriteBytesExt};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Cursor, Error, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::constants::{HASH_OFFSET, INDEX_FILE, PREFIX_SIZE};

#[derive(Debug)]
pub struct IndexEntry {
    pub ctime_sec: u32,
    pub ctime_nsec: u32,
    pub mtime_sec: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub hash: String,
    pub name: String,
    pub stage: u16,
}

pub fn create_directory(path: &str) -> Result<(), Error> {
    match fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            eprintln!("Oops, error while creating {}: {}", path, err);
            Err(err)
        }
    }
}

pub fn create_and_write_file(path: &str, content: &str) -> Result<(), Error> {
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

pub fn compress_and_store(input: &[u8], output_file: &str) -> Result<(), Error> {
    let file = File::create(output_file)?;
    let mut encoder = ZlibEncoder::new(file, Compression::default());
    encoder.write_all(input)?;
    encoder.finish()?;
    Ok(())
}

pub fn update_index(file_path: &str, hash: String) -> Result<(), Error> {
    let index_entry_result = create_index_entry(file_path, hash);
    println!("{:?}", index_entry_result);
    match index_entry_result {
        Ok(index_entry) => {
            let index_entry_bytes = encode_index_entry(&index_entry);
            save_to_index(&index_entry_bytes)?;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn create_index_entry(file: &str, hash: String) -> Result<IndexEntry, std::io::Error> {
    // Get file metadata
    let metadata = fs::metadata(file)?;

    // Extracting Unix-specific metadata
    let ctime_sec = metadata.ctime() as u32;
    let ctime_nsec = metadata.ctime_nsec() as u32;
    let mtime_sec = metadata.mtime() as u32;
    let mtime_nsec = metadata.mtime_nsec() as u32;

    // Calculate relative path
    let file_path = Path::new(file);
    let name = file_path.file_name().unwrap().to_str().unwrap().to_string();

    // Construct the IndexEntry struct
    let entry = IndexEntry {
        ctime_sec,
        ctime_nsec,
        mtime_sec,
        mtime_nsec,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        mode: 0o100644, // Regular file mode in Git index
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
        hash,
        name,
        stage: 0,
    };

    Ok(entry)
}

fn encode_index_entry(e: &IndexEntry) -> Vec<u8> {
    let mut buf = vec![0; PREFIX_SIZE];

    {
        let mut cursor = Cursor::new(&mut buf);

        cursor.write_u32::<BigEndian>(e.ctime_sec).unwrap();
        cursor.write_u32::<BigEndian>(e.ctime_nsec).unwrap();
        cursor.write_u32::<BigEndian>(e.mtime_sec).unwrap();
        cursor.write_u32::<BigEndian>(e.mtime_nsec).unwrap();
        cursor.write_u32::<BigEndian>(e.dev).unwrap();
        cursor.write_u32::<BigEndian>(e.ino).unwrap();
        cursor.write_u32::<BigEndian>(e.mode).unwrap();
        cursor.write_u32::<BigEndian>(e.uid).unwrap();
        cursor.write_u32::<BigEndian>(e.gid).unwrap();
        cursor.write_u32::<BigEndian>(e.size).unwrap();

        // Write flags and name length
        let name_length = if e.name.len() < 0xfff {
            e.name.len()
        } else {
            0xfff
        };
        let flags = (e.stage << 12) | name_length as u16;
        cursor.set_position(60);
        cursor.write_u16::<BigEndian>(flags).unwrap();
    }

    // Write the hash
    let hash_bytes = hex::decode(e.hash.to_string()).unwrap_or_else(|_| vec![0; 20]); // Assuming hash is hex encoded
    buf[HASH_OFFSET..HASH_OFFSET + hash_bytes.len()].copy_from_slice(&hash_bytes);

    // Write name
    let name_bytes = e.name.as_bytes();
    buf.extend_from_slice(name_bytes);

    // Calculate padding
    let padding_size = (8 - ((PREFIX_SIZE + name_bytes.len()) % 8)) % 8;
    if padding_size > 0 {
        let padding = vec![0; padding_size];
        buf.extend_from_slice(&padding);
    }

    buf
}

fn save_to_index(index_entry: &[u8]) -> Result<(), Error> {
    // Header constants
    const HEADER_SIGNATURE: [u8; 4] = [b'D', b'I', b'R', b'C'];
    const HEADER_VERSION: i32 = 2;
    const HEADER_INDEX_ENTRY: i32 = 1;

    // Create header buffer
    let mut header = vec![0u8; 12];
    header[..4].copy_from_slice(&HEADER_SIGNATURE);
    header[4..8].copy_from_slice(&HEADER_VERSION.to_be_bytes());
    header[8..12].copy_from_slice(&HEADER_INDEX_ENTRY.to_be_bytes());

    // Create buffer for index content
    let mut index_content = Vec::new();
    index_content.extend_from_slice(&header); // Append header
    index_content.extend_from_slice(index_entry); // Append encoded index entry

    // Create SHA-1 checksum
    let mut hasher = Sha1::new();
    hasher.update(&index_content);
    let checksum = hasher.finalize();

    // Combine index content and checksum
    let mut final_content = Vec::new();
    final_content.extend_from_slice(&index_content);
    final_content.extend_from_slice(&checksum);

    // Write to file
    let mut file = File::create(Path::new(INDEX_FILE))?;
    file.write_all(&final_content)?;

    Ok(())
}
