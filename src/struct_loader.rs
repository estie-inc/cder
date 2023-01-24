use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::{load_named_records, Dict};

/// StructLoader deserializes struct instances from specified file.
/// To resolve embedded tags, you need to provide HashMap that indicates corresponding records to
/// the labels specified in the yaml file.
///
/// NOTE: record names must be unique, otherwise the ealier records will be overwritten by the latter.
///
/// # Examples
/// ```rust
/// use serde::Deserialize;
/// use anyhow::Result;
/// 
/// // a model (struct)
/// #[derive(Deserialize, Clone)] // add this derive macro
/// struct User {
///   name: String,
///   email: String,
/// }
///
/// // a function that persists user record into users table
/// impl User {
///   // can be sync or async functions
///   async fn insert(input: &User) -> Result<(i64)> {
///     //
///     // this function inserts a corresponding User record into table,
///     // and returns its id when succeeded
///     //
///     # Ok(1)
///   }
/// }
///
/// // glue code you need to add
/// use cder::{ Dict, StructLoader };
///
/// # fn main() {
/// #     load_user("Peter");
/// # }
///
/// fn load_user(label: &str) -> Result<User> {
///     // provide your fixture filename followed by its directory
///     let mut loader = StructLoader::<User>::new("users.yml", "fixtures");
/// 
///     // deserializes User struct from the given fixture
///     // the argument is related to name resolution (described later)
///     let result = loader.load(&Dict::<String>::new())?;
///     result.get(label).map(|user| user.clone())
/// }
/// ```
pub struct StructLoader<T>
where
    T: DeserializeOwned,
{
    pub filename: String,
    pub base_dir: String,
    named_records: Option<Dict<T>>,
}

impl<T> StructLoader<T>
where
    T: DeserializeOwned,
{
    pub fn new(filename: &str, base_dir: &str) -> Self {
        Self {
            filename: filename.to_string(),
            base_dir: base_dir.to_string(),
            named_records: None,
        }
    }

    pub fn load(&mut self, dependencies: &Dict<String>) -> Result<&Self> {
        if self.named_records.is_some() {
            return Err(anyhow::anyhow!(
                "filename : {} the records have been loaded already",
                self.filename,
            ));
        }

        let records = load_named_records::<T>(&self.filename, &self.base_dir, dependencies)?;
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
