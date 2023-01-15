use anyhow::Result;
use std::{env, fs, path::PathBuf};

/// Read seeds from specified file
pub fn read_file(filename: &str, base_dir: Option<&str>) -> Result<String> {
    let mut path = match base_dir {
        Some(dir) => PathBuf::from(dir),
        None => {
            let mut path = env::current_dir().expect(
                "could not find curent_dir. you might want to check permission, or specify the base directory manually",
            );
            path.push("fixtures");
            path
        }
    };
    path.push(filename);

    fs::read_to_string(&path)
        .map_err(|err| anyhow::anyhow!("Can't open the file: {:?}\n   err: {}", path, err))
}
