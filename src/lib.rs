use anyhow::Result;
use log::info;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

mod reader;
use reader::read_file;

mod resolver;
use resolver::resolve_tags;

/// struct that contains deserialized records as well as its original file
/// internally HashMap is used to map records against their labelled names
/// NOTE: record names must be unique, otherwise the ealier records will be overwritten by the latter.
pub struct StructLoader<T>
where
    T: DeserializeOwned,
{
    filename: String,
    base_dir: Option<String>,
    record_map: Option<Dict<T>>,
}

pub type Dict<T> = HashMap<String, T>;

impl<T> StructLoader<T>
where
    T: DeserializeOwned,
{
    pub fn new(filename: &str, base_dir: Option<&str>) -> Self {
        Self {
            filename: filename.to_string(),
            base_dir: base_dir.map(|dir| dir.to_string()),
            record_map: None,
        }
    }

    pub fn load(&mut self, dependencies: &Dict<String>) -> Result<&Self> {
        info!("loading {}...", self.filename);

        if self.record_map.is_some() {
            return Err(anyhow::anyhow!(
                "filename : {} the records have been loaded already",
                self.filename,
            ));
        }

        // read contents as string from the seed file
        let raw_text = read_file(&self.filename, self.base_dir.as_deref())?;

        // replace embedded tags before deserialization gets started
        let parsed_text = resolve_tags(&raw_text, dependencies).map_err(|err| {
            anyhow::anyhow!(
                "failed to pre-process embedded tags: {}\n   err: {}",
                self.filename,
                err
            )
        })?;

        // deserialization
        // currently accepts yaml format only, but this could accept any other serde-compatible format, e.g. json
        let recoards = serde_yaml::from_str(&parsed_text).map_err(|err| {
            anyhow::anyhow!(
                "deserialization failed. check the file: {}
            err: {}",
                self.filename,
                err
            )
        })?;
        self.set_recoards(recoards)?;

        Ok(self)
    }

    pub fn get_all_records(&self) -> Result<&Dict<T>> {
        self.get_records()
    }

    pub fn get_record(&self, key: &str) -> Result<&T> {
        let records = self.get_records()?;
        records.get(key).ok_or_else(|| {
            anyhow::anyhow!(
                "{}: no record was found referred by the key: {}",
                self.filename,
                key,
            )
        })
    }

    fn set_recoards(&mut self, record_map: Dict<T>) -> Result<()> {
        if self.record_map.is_some() {
            return Err(anyhow::anyhow!(
                "filename : {} the records have been loaded already",
                self.filename,
            ));
        }

        self.record_map = Some(record_map);
        Ok(())
    }

    fn get_records(&self) -> Result<&Dict<T>> {
        self.record_map.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "filename : {} no records have been loaded yet",
                self.filename,
            )
        })
    }
}
