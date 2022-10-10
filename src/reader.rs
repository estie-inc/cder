use anyhow::Result;
use std::{env, fs, path::PathBuf};

/// Read seeds from specified file
pub fn read_seed_file(filename: &str) -> Result<String> {
    let fixture_dir = "fixtures"; // NOTE: consider overridiing this with a environment var
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(fixture_dir);
    path.push(filename);

    fs::read_to_string(&path)
        .map_err(|err| anyhow::anyhow!("Can't open the file: {:?}\n   err: {}", path, err))
}
