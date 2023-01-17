use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Item {
    pub name: String,
    pub price: f64,
}
#[derive(Deserialize, Clone)]
pub struct Customer {
    pub name: String,
    pub email: String,
    pub plan: Plan,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Plan {
    Premium,
    Family { shared_membership: u8 },
    Standard,
}
#[derive(Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub customer_id: i64,
    pub item_id: i64,
    pub quantity: i64,
    pub purchased_at: NaiveDateTime,
}
