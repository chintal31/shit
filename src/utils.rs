use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{Cursor, Error, ErrorKind, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

const INDEX_FILE: &str = ".git/index";

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

pub fn update_index(file_path: &str, hash: &str) -> Result<(), Error> {
    let file_metadata = fs::metadata(file_path)?;
    let file_name = Path::new(file_path).file_name().unwrap().to_str().unwrap();
    let file_name_bytes = file_name.as_bytes();
    let hash_bytes = hex::decode(hash).unwrap();

    let ctime = file_metadata
        .created()
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .as_secs();

    let mtime = file_metadata
        .modified()
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .as_secs();

    let ctime_nanos = file_metadata
        .created()
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .subsec_nanos();

    let mtime_nanos = file_metadata
        .modified()
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .subsec_nanos();

    let dev = file_metadata.dev();
    let ino = file_metadata.ino();
    let mode = file_metadata.mode();
    let uid = file_metadata.uid();
    let gid = file_metadata.gid();
    let file_size = file_metadata.len();

    let mut index_entry = Vec::new();
    index_entry.write_u32::<BigEndian>(ctime as u32)?;
    index_entry.write_u32::<BigEndian>(ctime_nanos)?;
    index_entry.write_u32::<BigEndian>(mtime as u32)?;
    index_entry.write_u32::<BigEndian>(mtime_nanos)?;
    index_entry.write_u32::<BigEndian>(dev as u32)?;
    index_entry.write_u32::<BigEndian>(ino as u32)?;
    index_entry.write_u32::<BigEndian>(mode as u32)?;
    index_entry.write_u32::<BigEndian>(uid as u32)?;
    index_entry.write_u32::<BigEndian>(gid as u32)?;
    index_entry.write_u32::<BigEndian>(file_size as u32)?;
    index_entry.extend_from_slice(&hash_bytes);
    index_entry.write_u16::<BigEndian>(file_name_bytes.len() as u16 + 1)?; // including null byte
    index_entry.extend_from_slice(file_name_bytes);
    index_entry.push(0); // Null terminator

    // Calculate padding to align to 8 bytes
    let entry_size = index_entry.len() + 2; // Add 2 for file_name_len (16-bit)
    let padding_length = (8 - (entry_size % 8)) % 8;
    index_entry.extend(vec![0; padding_length]);

    // Read existing index file content if it exists
    let mut index_content = Vec::new();
    if let Ok(mut index_file) = File::open(INDEX_FILE) {
        index_file.read_to_end(&mut index_content)?;
    }

    // Update the index file content
    let mut index_file = File::create(INDEX_FILE)?;
    if index_content.is_empty() {
        // Write the header for a new index file
        index_file.write_all(b"DIRC")?;
        index_file.write_u32::<BigEndian>(2)?; // Index version
        index_file.write_u32::<BigEndian>(1)?; // Number of index entries
    } else {
        // Update the number of entries in the existing index
        let mut entry_count_cursor = Cursor::new(&mut index_content[8..12]);
        let current_entry_count = entry_count_cursor.read_u32::<BigEndian>()?;
        entry_count_cursor.set_position(0);
        entry_count_cursor.write_u32::<BigEndian>(current_entry_count + 1)?;
        index_file.write_all(&index_content)?;
    }

    // Append the new index entry
    index_file.write_all(&index_entry)?;

    Ok(())
}
