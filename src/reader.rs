use anyhow::Result;
use std::{env, fs, path::PathBuf};

/// Read seeds from specified file
pub fn read_file(filename: &str, base_dir: &str) -> Result<String> {
    let path = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(base_dir)
        .join(filename);

    fs::read_to_string(&path)
        .map_err(|err| anyhow::anyhow!("Can't open the file: {:?}\n   err: {}", path, err))
}
