use crate::{load_named_records, Dict};
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::future::Future;
/// DatabaseSeeder persists data deserialized from specified file.
/// Internally it keeps record label mapped against its id on insertion. The mapping can be reused
/// later process to resolve embedded tags.
///
/// NOTE: record names must be unique, otherwise the ealier records will be overwritten by the latter.
///
/// # Examples
/// ```rust
/// use serde::Deserialize;
/// use anyhow::Result;
///
/// // a model (struct)
/// #[derive(Deserialize)] // add this derive macro
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
/// use cder::DatabaseSeeder;
///
/// # fn main() {
/// #     populate_seeds();
/// # }
///
/// async fn populate_seeds() -> Result<()> {
///     let mut seeder = DatabaseSeeder::new();
///
///     seeder
///         .populate_async("fixtures/users.yml", |input| {
///             async move { User::insert(&input).await }
///         })
///         .await?;
///
///     Ok(())
/// }
/// ```

pub struct DatabaseSeeder {
    pub filenames: Vec<String>,
    pub base_dir: String,
    name_resolver: Dict<String>,
}

impl Default for DatabaseSeeder {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseSeeder {
    pub fn new() -> Self {
        Self {
            filenames: Vec::new(),
            base_dir: String::new(),
            name_resolver: Dict::<String>::new(),
        }
    }

    pub fn set_dir(&mut self, base_dir: &str) {
        self.base_dir = base_dir.to_string();
    }

    /// ```rust
    /// use cder::DatabaseSeeder;
    /// # use serde::Deserialize;
    /// # use anyhow::Result;
    /// #
    /// # #[derive(Deserialize)] // add this derive macro
    /// # struct User {
    /// #   name: String,
    /// #   email: String,
    /// # }
    /// #
    /// # impl User {
    /// #   fn insert(input: &User) -> Result<(i64)> {
    /// #     //
    /// #     // this function inserts a corresponding User record into table,
    /// #     // and returns its id when succeeded
    /// #     //
    /// #     Ok(1)
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// #     populate_seeds();
    /// # }
    ///
    /// async fn populate_seeds() -> Result<()> {
    ///     let mut seeder = DatabaseSeeder::new();
    ///
    /// seeder
    ///     .populate("fixures/users.yml", |input| {
    ///         // this block can contain any non-async functions
    ///         // but it has to return Result<i64> in the end
    ///         User::insert(&input)
    ///     });
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn populate<F, T, U>(&mut self, filename: &str, mut loader: F) -> Result<Vec<U>>
    where
        F: FnMut(T) -> Result<U>,
        T: DeserializeOwned,
        U: ToString,
    {
        let named_records = load_named_records::<T>(filename, &self.base_dir, &self.name_resolver)?;
        let mut ids = Vec::new();

        for (name, record) in named_records {
            let id = loader(record)?;
            self.name_resolver.insert(name.clone(), id.to_string());
            ids.push(id);
        }
        Ok(ids)
    }

    /// ```rust
    /// use cder::DatabaseSeeder;
    /// # use serde::Deserialize;
    /// # use anyhow::Result;
    /// #
    /// # #[derive(Deserialize)] // add this derive macro
    /// # struct User {
    /// #   name: String,
    /// #   email: String,
    /// # }
    /// #
    /// # impl User {
    /// #   async fn insert(input: &User) -> Result<(i64)> {
    /// #     //
    /// #     // this function inserts a corresponding User record into table,
    /// #     // and returns its id when succeeded
    /// #     //
    /// #     Ok(1)
    /// #   }
    /// # }
    /// #
    /// # fn main() {
    /// #     populate_seeds();
    /// # }
    ///
    /// async fn populate_seeds() -> Result<()> {
    ///     let mut seeder = DatabaseSeeder::new();
    ///
    ///     seeder
    ///         .populate_async("fixtures/users.yml", |input| {
    ///             async move { User::insert(&input).await }
    ///         })
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn populate_async<Fut, F, T, U>(
        &mut self,
        filename: &str,
        mut loader: F,
    ) -> Result<Vec<U>>
    where
        Fut: Future<Output = Result<U>>,
        F: FnMut(T) -> Fut,
        T: DeserializeOwned,
        U: ToString,
    {
        let named_records = load_named_records::<T>(filename, &self.base_dir, &self.name_resolver)?;
        self.filenames.push(filename.to_string());

        let mut ids = Vec::new();

        for (name, record) in named_records {
            let id = loader(record).await?;
            self.name_resolver.insert(name.clone(), id.to_string());
            ids.push(id);
        }
        Ok(ids)
    }
}
