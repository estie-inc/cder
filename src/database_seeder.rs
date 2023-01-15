use crate::{load_named_records, Dict};
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;

pub struct DatabaseSeeder {
    pub filenames: Vec<String>,
    pub base_dir: Option<String>,
    name_resolver: Dict<String>,
}

impl Default for DatabaseSeeder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DatabaseSeeder {
    pub fn new(base_dir: Option<&str>) -> Self {
        Self {
            filenames: Vec::new(),
            base_dir: base_dir.map(|dir| dir.to_string()),
            name_resolver: Dict::<String>::new(),
        }
    }

    pub fn populate<F, T>(&mut self, filename: &str, mut loader: F) -> Result<Vec<i64>>
    where
        F: FnMut(T) -> Result<i64>,
        T: DeserializeOwned,
    {
        let named_records =
            load_named_records::<T>(filename, self.base_dir.as_deref(), &self.name_resolver)?;
        let mut ids = Vec::new();

        for (name, record) in named_records {
            let id = loader(record)?;
            self.name_resolver.insert(name.clone(), id.to_string());
            ids.push(id);
        }
        Ok(ids)
    }

    /// experimental
    pub async fn populate_async<'a, F, T>(
        &mut self,
        filename: &str,
        mut loader: F,
    ) -> Result<Vec<i64>>
    where
        // XXX: The type of return value F should include +Send, but it brings in higher-ranked
        // lifetime error with the caller blocks.
        // (futures::future::BoxFuture is not available for the same reason)
        // related to this issue?
        // https://github.com/rust-lang/rust/issues/102211
        F: FnMut(T) -> Pin<Box<dyn Future<Output = Result<i64>> + 'a>>,
        T: DeserializeOwned + 'a,
    {
        let named_records =
            load_named_records::<T>(filename, self.base_dir.as_deref(), &self.name_resolver)?;
        let mut ids = Vec::new();

        for (name, record) in named_records {
            let id = loader(record).await?;
            self.name_resolver.insert(name.clone(), id.to_string());
            ids.push(id);
        }
        Ok(ids)
    }
}
