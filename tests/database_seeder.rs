mod test_utils;
use test_utils::*;
extern crate cder;

use anyhow::Result;
use cder::DatabaseSeeder;
use tokio::runtime::Runtime;

#[test]
fn test_database_seeder_new() {
    let seeder = DatabaseSeeder::new(Some("fixtures"));
    assert!(seeder.filenames.is_empty());
    assert_eq!(seeder.base_dir, Some("fixtures".to_string()));
}

#[test]
fn test_database_seeder_populate_items() -> Result<()> {
    let base_dir = get_test_base_dir();
    let mut mock_table = MockTable::<Item>::new(vec![
        ("melon".to_string(), 1),
        ("orange".to_string(), 2),
        ("apple".to_string(), 3),
        ("carrot".to_string(), 4),
    ]);
    let rt = Runtime::new().unwrap();

    let mut seeder = DatabaseSeeder::new(base_dir.as_deref());
    let ids = seeder.populate("items.yml", |input: Item| {
        rt.block_on(mock_table.insert(input))
    })?;

    let persisted_records = mock_table.records;
    let records = sort_records_by_ids(persisted_records, ids);

    assert_eq!(records[0].name, "melon");
    assert_eq!(records[0].price, 500.0);

    assert_eq!(records[1].name, "orange");
    assert_eq!(records[1].price, 200.0);

    assert_eq!(records[2].name, "apple");
    assert_eq!(records[2].price, 100.0);

    assert_eq!(records[3].name, "carrot");
    assert_eq!(records[3].price, 150.0);

    Ok(())
}

#[tokio::test]
async fn test_database_seeder_populate_async_items() -> Result<()> {
    // TODO: test against populate_async
    // currently MockTable is not available within async blocks
    // (captured mutable is not allowed to escape from the closure)
    // Instead of this mock struct, database might be needed for testing purpose

    // let base_dir = get_test_base_dir();
    // let mut mock_table = MockTable::<Item>::new(vec![
    //     ("melon".to_string(), 1),
    //     ("orange".to_string(), 2),
    //     ("apple".to_string(), 3),
    //     ("carrot".to_string(), 4),
    // ]);

    // let mut seeder = DatabaseSeeder::new(base_dir.as_deref());
    // let ids = seeder
    //     .populate_async("items.yml", |input: Item| {
    //         let mock_table = &mut mock_table;
    //         Box::pin(async move { mock_table.insert(input).await })
    //     })
    //     .await?;

    // let persisted_records = mock_table.records;
    // let records = sort_records_by_ids(persisted_records, ids);

    // assert_eq!(records[0].name, "melon");
    // assert_eq!(records[0].price, 500.0);

    // assert_eq!(records[1].name, "orange");
    // assert_eq!(records[1].price, 200.0);

    // assert_eq!(records[2].name, "apple");
    // assert_eq!(records[2].price, 100.0);

    // assert_eq!(records[3].name, "carrot");
    // assert_eq!(records[3].price, 150.0);

    Ok(())
}

#[test]
fn test_database_seeder_populate_customers() -> Result<()> {
    let base_dir = get_test_base_dir();
    let mut mock_table = MockTable::<Customer>::new(vec![
        ("Alice".to_string(), 1),
        ("Bob".to_string(), 2),
        ("Developer".to_string(), 3),
    ]);
    let rt = Runtime::new().unwrap();

    let mut seeder = DatabaseSeeder::new(base_dir.as_deref());
    let ids = seeder.populate("customers.yml", |input: Customer| {
        rt.block_on(mock_table.insert(input))
    })?;

    let persisted_records = mock_table.records;
    let records = sort_records_by_ids(persisted_records, ids);

    assert_eq!(records[0].name, "Alice");
    assert_eq!(records[0].email, "alice@example.com");
    assert_eq!(records[0].plan, Plan::Premium);

    assert_eq!(records[1].name, "Bob");
    assert_eq!(records[1].email, "bob@example.com");
    assert_eq!(
        records[1].plan,
        Plan::Family {
            shared_membership: 4
        }
    );

    assert_eq!(records[2].name, "Developer");
    // falls back to default
    assert_eq!(records[2].email, "developer@example.com");
    assert_eq!(records[2].plan, Plan::Standard);

    Ok(())
}

#[test]
fn test_database_seeder_populate_orders() -> Result<()> {
    let base_dir = get_test_base_dir();
    let rt = Runtime::new().unwrap();

    let mut seeder = DatabaseSeeder::new(base_dir.as_deref());

    {
        // when dependencies are missing

        let mut mock_orders_table = MockTable::<Order>::new(vec![
            ("1200".to_string(), 1),
            ("1201".to_string(), 2),
            ("1202".to_string(), 3),
            ("1203".to_string(), 3),
        ]);
        let results = seeder.populate("orders.yml", |input: Order| {
            rt.block_on(mock_orders_table.insert(input))
        });

        assert!(results.is_err());
    }

    {
        // when dependencies are provided
        let mut mock_items_table = MockTable::<Item>::new(vec![
            ("melon".to_string(), 1),
            ("orange".to_string(), 2),
            ("apple".to_string(), 3),
            ("carrot".to_string(), 4),
        ]);
        seeder.populate("items.yml", |input: Item| {
            rt.block_on(mock_items_table.insert(input))
        })?;
        let mut mock_customers_table = MockTable::<Customer>::new(vec![
            ("Alice".to_string(), 1),
            ("Bob".to_string(), 2),
            ("Developer".to_string(), 3),
        ]);
        seeder.populate("customers.yml", |input: Customer| {
            rt.block_on(mock_customers_table.insert(input))
        })?;

        let mut mock_orders_table = MockTable::<Order>::new(vec![
            ("1200".to_string(), 1),
            ("1201".to_string(), 2),
            ("1202".to_string(), 3),
            ("1203".to_string(), 4),
        ]);
        let ids = seeder.populate("orders.yml", |input: Order| {
            rt.block_on(mock_orders_table.insert(input))
        })?;

        let persisted_records = mock_orders_table.records;
        let records = sort_records_by_ids(persisted_records, ids);

        assert_eq!(records[0].customer_id, 1);
        assert_eq!(records[0].item_id, 3);
        assert_eq!(records[0].quantity, 2);
        assert_eq!(
            records[0].purchased_at,
            parse_datetime("2021-03-01 15:15:44")?
        );

        assert_eq!(records[1].customer_id, 2);
        assert_eq!(records[1].item_id, 1);
        assert_eq!(records[1].quantity, 1);
        assert_eq!(
            records[1].purchased_at,
            parse_datetime("2021-03-02 07:51:20")?
        );

        assert_eq!(records[2].customer_id, 1);
        assert_eq!(records[2].item_id, 4);
        assert_eq!(records[2].quantity, 4);
        assert_eq!(
            records[2].purchased_at,
            parse_datetime("2021-03-10 10:10:33")?
        );

        assert_eq!(records[3].customer_id, 3);
        assert_eq!(records[3].item_id, 1);
        assert_eq!(records[3].quantity, 2);
        assert_eq!(
            records[3].purchased_at,
            parse_datetime("2021-03-11 11:55:44")?
        );
    }

    Ok(())
}
