mod test_utils;
use test_utils::*;
extern crate cder;

use anyhow::Result;
use cder::DatabaseSeeder;

#[test]
fn test_database_seeder_new() {
    let seeder = DatabaseSeeder::new("fixtures");
    assert!(seeder.filenames.is_empty());
    assert_eq!(seeder.base_dir, "fixtures".to_string());
}

#[tokio::test]
async fn test_database_seeder_populate_async_items() -> Result<()> {
    let base_dir = get_test_base_dir();
    let mock_table = MockTable::<Item>::new(vec![
        ("melon".to_string(), 1),
        ("orange".to_string(), 2),
        ("apple".to_string(), 3),
        ("carrot".to_string(), 4),
    ]);

    let mut seeder = DatabaseSeeder::new(&base_dir);
    let ids = seeder
        .populate_async("items.yml", |input: Item| {
            let mut mock_table = mock_table.clone();
            Box::pin(async move { mock_table.insert(input).await })
        })
        .await?;

    let persisted_records = mock_table.get_records();
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
async fn test_database_seeder_populate_async_customers() -> Result<()> {
    let base_dir = get_test_base_dir();
    let mock_table = MockTable::<Customer>::new(vec![
        ("Alice".to_string(), 1),
        ("Bob".to_string(), 2),
        ("Developer".to_string(), 3),
    ]);

    let mut seeder = DatabaseSeeder::new(&base_dir);
    let ids = seeder
        .populate_async("customers.yml", |input: Customer| {
            let mut mock_table = mock_table.clone();
            Box::pin(async move { mock_table.insert(input).await })
        })
        .await?;

    let persisted_records = mock_table.get_records();
    let records = sort_records_by_ids(persisted_records, ids);

    assert_eq!(records[0].name, "Alice");
    assert_eq!(records[0].emails.len(), 1);
    assert_eq!(records[0].emails[0], "alice@example.com");
    assert_eq!(records[0].plan, Plan::Premium);
    assert_eq!(records[0].country_code, None);

    assert_eq!(records[1].name, "Bob");
    assert_eq!(records[1].emails.len(), 2);
    assert_eq!(records[1].emails[0], "bob@example.com");
    assert_eq!(records[1].emails[1], "bob.doe@example.co.jp");
    assert_eq!(
        records[1].plan,
        Plan::Family {
            shared_membership: 4
        }
    );
    assert_eq!(records[1].country_code, Some(81));

    assert_eq!(records[2].name, "Developer");
    assert_eq!(records[2].emails.len(), 1);
    // falls back to default
    assert_eq!(records[2].emails[0], "developer@example.com");
    assert_eq!(records[2].plan, Plan::Standard);
    assert_eq!(records[2].country_code, Some(44));

    Ok(())
}

#[tokio::test]
async fn test_database_seeder_populate_async_orders() -> Result<()> {
    let base_dir = get_test_base_dir();
    let mut seeder = DatabaseSeeder::new(&base_dir);

    {
        // when dependencies are missing

        let mock_orders_table = MockTable::<Order>::new(vec![
            ("1200".to_string(), 1),
            ("1201".to_string(), 2),
            ("1202".to_string(), 3),
            ("1203".to_string(), 4),
        ]);
        let results = seeder
            .populate_async("orders.yml", |input: Order| {
                let mut mock_orders_table = mock_orders_table.clone();
                Box::pin(async move { mock_orders_table.insert(input).await })
            })
            .await;

        assert!(results.is_err());
    }

    {
        // when dependencies are provided
        let mock_items_table = MockTable::<Item>::new(vec![
            ("melon".to_string(), 1),
            ("orange".to_string(), 2),
            ("apple".to_string(), 3),
            ("carrot".to_string(), 4),
        ]);
        seeder
            .populate_async("items.yml", |input: Item| {
                let mut mock_items_table = mock_items_table.clone();
                Box::pin(async move { mock_items_table.insert(input).await })
            })
            .await?;
        let mock_customers_table = MockTable::<Customer>::new(vec![
            ("Alice".to_string(), 1),
            ("Bob".to_string(), 2),
            ("Developer".to_string(), 3),
        ]);
        seeder
            .populate_async("customers.yml", |input: Customer| {
                let mut mock_customers_table = mock_customers_table.clone();
                Box::pin(async move { mock_customers_table.insert(input).await })
            })
            .await?;

        let mock_orders_table = MockTable::<Order>::new(vec![
            ("1200".to_string(), 1),
            ("1201".to_string(), 2),
            ("1202".to_string(), 3),
            ("1203".to_string(), 4),
        ]);
        let ids = seeder
            .populate_async("orders.yml", |input: Order| {
                let mut mock_orders_table = mock_orders_table.clone();
                Box::pin(async move { mock_orders_table.insert(input).await })
            })
            .await?;

        let persisted_records = mock_orders_table.get_records();
        let records = sort_records_by_ids(persisted_records, ids);

        assert_eq!(records[0].id, 1200);
        assert_eq!(records[0].customer_id, 1);
        assert_eq!(records[0].item_id, 3);
        assert_eq!(records[0].quantity, 2);
        assert_eq!(
            records[0].purchased_at,
            parse_datetime("2021-03-01 15:15:44")?
        );

        assert_eq!(records[1].id, 1201);
        assert_eq!(records[1].customer_id, 2);
        assert_eq!(records[1].item_id, 1);
        assert_eq!(records[1].quantity, 1);
        assert_eq!(
            records[1].purchased_at,
            parse_datetime("2021-03-02 07:51:20")?
        );

        assert_eq!(records[2].id, 1202);
        assert_eq!(records[2].customer_id, 1);
        assert_eq!(records[2].item_id, 4);
        assert_eq!(records[2].quantity, 4);
        assert_eq!(
            records[2].purchased_at,
            parse_datetime("2021-03-10 10:10:33")?
        );

        assert_eq!(records[3].id, 1203);
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
