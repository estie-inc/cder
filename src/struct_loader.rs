use anyhow::Result;
use log::info;
use serde::de::DeserializeOwned;

use crate::{load_named_records, Dict};

/// struct that contains deserialized records as well as its original file
/// internally HashMap is used to map records against their labelled names
/// NOTE: record names must be unique, otherwise the ealier records will be overwritten by the latter.
pub struct StructLoader<T>
where
    T: DeserializeOwned,
{
    pub filename: String,
    pub base_dir: Option<String>,
    named_records: Option<Dict<T>>,
}

impl<T> StructLoader<T>
where
    T: DeserializeOwned,
{
    pub fn new(filename: &str, base_dir: Option<&str>) -> Self {
        Self {
            filename: filename.to_string(),
            base_dir: base_dir.map(|dir| dir.to_string()),
            named_records: None,
        }
    }

    pub fn load(&mut self, dependencies: &Dict<String>) -> Result<&Self> {
        info!("loading {}...", self.filename);

        if self.named_records.is_some() {
            return Err(anyhow::anyhow!(
                "filename : {} the records have been loaded already",
                self.filename,
            ));
        }

        let records =
            load_named_records::<T>(&self.filename, self.base_dir.as_deref(), dependencies)?;
        self.set_recoards(records)?;

        Ok(self)
    }

    pub fn get(&self, key: &str) -> Result<&T> {
        let records = self.get_records()?;
        records.get(key).ok_or_else(|| {
            anyhow::anyhow!(
                "{}: no record was found referred by the key: {}",
                self.filename,
                key,
            )
        })
    }

    pub fn get_all_records(&self) -> Result<&Dict<T>> {
        self.get_records()
    }

    fn set_recoards(&mut self, named_records: Dict<T>) -> Result<()> {
        if self.named_records.is_some() {
            return Err(anyhow::anyhow!(
                "filename : {} the records have been loaded already",
                self.filename,
            ));
        }

        self.named_records = Some(named_records);
        Ok(())
    }

    fn get_records(&self) -> Result<&Dict<T>> {
        self.named_records.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "filename : {} no records have been loaded yet",
                self.filename,
            )
        })
    }
}
