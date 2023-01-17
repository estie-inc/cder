mod mock_database;
mod types;
pub use mock_database::{sort_records_by_ids, MockTable};
pub use types::{Customer, Item, Order, Plan};

use anyhow::Result;
use chrono::NaiveDateTime;
use std::env;

pub fn get_test_base_dir() -> Option<String> {
    let mut path = env::current_dir().unwrap();
    path.push("tests/fixtures");
    path.to_str().map(|s| s.to_string())
}

pub fn parse_datetime(s: &str) -> Result<NaiveDateTime> {
    let datetime = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    Ok(datetime)
}
