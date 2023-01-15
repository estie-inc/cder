use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::{collections::HashMap, env};

#[derive(Deserialize)]
pub struct Item {
    pub name: String,
    pub price: f64,
}
#[derive(Deserialize)]
pub struct Customer {
    pub name: String,
    pub email: String,
    pub plan: Plan,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum Plan {
    Premium,
    Family { shared_membership: u8 },
    Standard,
}
#[derive(Deserialize)]
pub struct Order {
    pub id: i64,
    pub customer_id: i64,
    pub item_id: i64,
    pub quantity: i64,
    pub purchased_at: NaiveDateTime,
}

pub fn get_test_base_dir() -> Option<String> {
    let mut path = env::current_dir().unwrap();
    path.push("tests/fixtures");
    path.to_str().map(|s| s.to_string())
}

pub fn parse_datetime(s: &str) -> Result<NaiveDateTime> {
    let datetime = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")?;
    Ok(datetime)
}

pub struct MockTable<T> {
    ids_by_name: HashMap<String, i64>,
    pub records: Vec<T>,
}
