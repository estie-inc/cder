use anyhow::Result;
use std::{fs, path::PathBuf};

/// Read seeds from specified file
pub fn read_file(filename: &str, base_dir: &str) -> Result<String> {
    let mut path = PathBuf::from(base_dir);
    path.push(filename);

    fs::read_to_string(&path)
        .map_err(|err| anyhow::anyhow!("Can't open the file: {:?}\n   err: {}", path, err))
}
