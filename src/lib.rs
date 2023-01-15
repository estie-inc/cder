mod reader;
mod resolver;
mod struct_loader;
pub use struct_loader::StructLoader;

use anyhow::Result;
use reader::read_file;
use resolver::resolve_tags;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub type Dict<T> = HashMap<String, T>;

fn load_named_records<T>(
    filename: &str,
    base_dir: Option<&str>,
    dependencies: &Dict<String>,
) -> Result<Dict<T>>
where
    T: DeserializeOwned,
{
    // read contents as string from the seed file
    let raw_text = read_file(filename, base_dir)?;

    // replace embedded tags before deserialization gets started
    let parsed_text = resolve_tags(&raw_text, dependencies).map_err(|err| {
        anyhow::anyhow!(
            "failed to pre-process embedded tags: {}\n   err: {}",
            filename,
            err
        )
    })?;

    // deserialization
    // currently accepts yaml format only, but this could accept any other serde-compatible format, e.g. json
    let records = serde_yaml::from_str(&parsed_text).map_err(|err| {
        anyhow::anyhow!(
            "deserialization failed. check the file: {}
            err: {}",
            filename,
            err
        )
    })?;

    Ok(records)
}
