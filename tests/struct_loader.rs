use anyhow::Result;
use cder::{Dict, StructLoader};
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::env;

extern crate cder;

#[derive(Deserialize)]
struct Item {
    name: String,
    price: f64,
}
#[derive(Deserialize)]
struct Customer {
    name: String,
    email: String,
    plan: Plan,
}

#[derive(Deserialize, Debug, PartialEq)]
enum Plan {
    Premium,
    Family { shared_membership: u8 },
    Standard,
}
#[derive(Deserialize)]
struct Order {
    customer_id: i64,
    item_id: i64,
    quantity: i64,
    purchased_at: NaiveDateTime,
}

fn get_test_base_dir() -> Option<String> {
    let mut path = env::current_dir().unwrap();
    path.push("tests/fixtures");
    path.to_str().map(|s| s.to_string())
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime> {
    let datetime = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")?;
    Ok(datetime)
}

#[test]
fn test_struct_loader_new() {
    let loader = StructLoader::<Item>::new("items.yml", Some("fixtures"));
    assert_eq!(loader.filename, "items.yml");
    assert_eq!(loader.base_dir, Some("fixtures".to_string()));
}

#[test]
fn test_struct_loader_load_items() -> Result<()> {
    let empty_dict = Dict::<String>::new();
    let base_dir = get_test_base_dir();

    let mut loader = StructLoader::<Item>::new("items.yml", base_dir.as_deref());
    loader.load(&empty_dict)?;

    let item = loader.get("Melon")?;
    assert_eq!(item.name, "melon");
    assert_eq!(item.price, 500.0);

    let item = loader.get("Orange")?;
    assert_eq!(item.name, "orange");
    assert_eq!(item.price, 200.0);

    let item = loader.get("Apple")?;
    assert_eq!(item.name, "apple");
    assert_eq!(item.price, 100.0);

    let item = loader.get("Carrot")?;
    assert_eq!(item.name, "carrot");
    assert_eq!(item.price, 150.0);

    Ok(())
}

#[test]
fn test_struct_loader_get_all_items() -> Result<()> {
    let empty_dict = Dict::<String>::new();
    let base_dir = get_test_base_dir();

    let mut loader = StructLoader::<Item>::new("items.yml", base_dir.as_deref());
    loader.load(&empty_dict)?;

    let named_records = loader.get_all_records()?;

    let item = named_records.get("Melon").unwrap();
    assert_eq!(item.name, "melon");
    assert_eq!(item.price, 500.0);

    let item = named_records.get("Orange").unwrap();
    assert_eq!(item.name, "orange");
    assert_eq!(item.price, 200.0);

    let item = named_records.get("Apple").unwrap();
    assert_eq!(item.name, "apple");
    assert_eq!(item.price, 100.0);

    let item = named_records.get("Carrot").unwrap();
    assert_eq!(item.name, "carrot");
    assert_eq!(item.price, 150.0);

    Ok(())
}

#[test]
fn test_struct_loader_load_customers() -> Result<()> {
    let empty_dict = Dict::<String>::new();
    let base_dir = get_test_base_dir();

    {
        // when ENV var is specified

        env::set_var("DEV_EMAIL", "johndoo@dev.example.com");
        let mut loader = StructLoader::<Customer>::new("customers.yml", base_dir.as_deref());
        loader.load(&empty_dict)?;

        let costomer = loader.get("Alice")?;
        assert_eq!(costomer.name, "Alice");
        assert_eq!(costomer.email, "alice@example.com");
        assert_eq!(costomer.plan, Plan::Premium);

        let costomer = loader.get("Bob")?;
        assert_eq!(costomer.name, "Bob");
        assert_eq!(costomer.email, "bob@example.com");
        assert_eq!(
            costomer.plan,
            Plan::Family {
                shared_membership: 4
            }
        );

        let costomer = loader.get("Dev")?;
        assert_eq!(costomer.name, "Developer");
        // replaced by the env var
        assert_eq!(costomer.email, "johndoo@dev.example.com");
        assert_eq!(costomer.plan, Plan::Standard);

        // teardown
        env::remove_var("DEV_EMAIL");
    }

    {
        // when ENV var is not specified

        let mut loader = StructLoader::<Customer>::new("customers.yml", base_dir.as_deref());
        loader.load(&empty_dict)?;

        let costomer = loader.get("Alice")?;
        assert_eq!(costomer.name, "Alice");
        assert_eq!(costomer.email, "alice@example.com");
        assert_eq!(costomer.plan, Plan::Premium);

        let costomer = loader.get("Bob")?;
        assert_eq!(costomer.name, "Bob");
        assert_eq!(costomer.email, "bob@example.com");
        assert_eq!(
            costomer.plan,
            Plan::Family {
                shared_membership: 4
            }
        );

        let costomer = loader.get("Dev")?;
        assert_eq!(costomer.name, "Developer");
        // falls back to default
        assert_eq!(costomer.email, "developer@example.com");
        assert_eq!(costomer.plan, Plan::Standard);
    }

    Ok(())
}

#[test]
fn test_struct_loader_load_orders() -> Result<()> {
    let base_dir = get_test_base_dir();
    let empty_dict = Dict::<String>::new();

    {
        // when dependencies are missing

        let mut loader = StructLoader::<Order>::new("orders.yml", base_dir.as_deref());
        let result = loader.load(&empty_dict);

        assert!(result.is_err());
    }

    {
        // when dependencies are provided
        let foreign_keys = vec![
            ("Alice", 1),
            ("Bob", 2),
            ("Dev", 3),
            ("Melon", 100),
            ("Orange", 101),
            ("Apple", 102),
            ("Carrot", 103),
        ];
        let mapping = foreign_keys
            .into_iter()
            .map(|(name, id)| (name.to_string(), id.to_string()))
            .collect::<Dict<String>>();

        let mut loader = StructLoader::<Order>::new("orders.yml", base_dir.as_deref());
        loader.load(&mapping)?;

        let order = loader.get("Order1")?;
        assert_eq!(order.customer_id, 1);
        assert_eq!(order.item_id, 102);
        assert_eq!(order.quantity, 2);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-01 15:15:44")?);

        let order = loader.get("Order2")?;
        assert_eq!(order.customer_id, 2);
        assert_eq!(order.item_id, 100);
        assert_eq!(order.quantity, 1);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-02 07:51:20")?);

        let order = loader.get("Order3")?;
        assert_eq!(order.customer_id, 1);
        assert_eq!(order.item_id, 103);
        assert_eq!(order.quantity, 4);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-10 10:10:33")?);

        let order = loader.get("Order4")?;
        assert_eq!(order.customer_id, 3);
        assert_eq!(order.item_id, 100);
        assert_eq!(order.quantity, 2);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-11 11:55:44")?);
    }

    Ok(())
}
