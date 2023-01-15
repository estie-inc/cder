mod test_utils;
use test_utils::*;
extern crate cder;

use anyhow::Result;
use cder::{Dict, StructLoader};
use std::env;

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

        let customer = loader.get("Alice")?;
        assert_eq!(customer.name, "Alice");
        assert_eq!(customer.email, "alice@example.com");
        assert_eq!(customer.plan, Plan::Premium);

        let customer = loader.get("Bob")?;
        assert_eq!(customer.name, "Bob");
        assert_eq!(customer.email, "bob@example.com");
        assert_eq!(
            customer.plan,
            Plan::Family {
                shared_membership: 4
            }
        );

        let customer = loader.get("Dev")?;
        assert_eq!(customer.name, "Developer");
        // replaced by the env var
        assert_eq!(customer.email, "johndoo@dev.example.com");
        assert_eq!(customer.plan, Plan::Standard);

        // teardown
        env::remove_var("DEV_EMAIL");
    }

    {
        // when ENV var is not specified

        let mut loader = StructLoader::<Customer>::new("customers.yml", base_dir.as_deref());
        loader.load(&empty_dict)?;

        let customer = loader.get("Alice")?;
        assert_eq!(customer.name, "Alice");
        assert_eq!(customer.email, "alice@example.com");
        assert_eq!(customer.plan, Plan::Premium);

        let customer = loader.get("Bob")?;
        assert_eq!(customer.name, "Bob");
        assert_eq!(customer.email, "bob@example.com");
        assert_eq!(
            customer.plan,
            Plan::Family {
                shared_membership: 4
            }
        );

        let customer = loader.get("Dev")?;
        assert_eq!(customer.name, "Developer");
        // falls back to default
        assert_eq!(customer.email, "developer@example.com");
        assert_eq!(customer.plan, Plan::Standard);
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
        assert_eq!(order.id, 1200);
        assert_eq!(order.customer_id, 1);
        assert_eq!(order.item_id, 102);
        assert_eq!(order.quantity, 2);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-01 15:15:44")?);

        let order = loader.get("Order2")?;
        assert_eq!(order.id, 1201);
        assert_eq!(order.customer_id, 2);
        assert_eq!(order.item_id, 100);
        assert_eq!(order.quantity, 1);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-02 07:51:20")?);

        let order = loader.get("Order3")?;
        assert_eq!(order.id, 1202);
        assert_eq!(order.customer_id, 1);
        assert_eq!(order.item_id, 103);
        assert_eq!(order.quantity, 4);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-10 10:10:33")?);

        let order = loader.get("Order4")?;
        assert_eq!(order.id, 1203);
        assert_eq!(order.customer_id, 3);
        assert_eq!(order.item_id, 100);
        assert_eq!(order.quantity, 2);
        assert_eq!(order.purchased_at, parse_datetime("2021-03-11 11:55:44")?);
    }

    Ok(())
}
