[![cder](https://github.com/estie-inc/cder/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/estie-inc/cder/actions/workflows/ci.yml)
[![Latest version](https://img.shields.io/crates/v/cder.svg)](https://crates.io/crates/cder)
[![Documentation](https://docs.rs/cder/badge.svg)](https://docs.rs/cder)
![licence](https://img.shields.io/github/license/estie-inc/cder)

# cder


### A lightweight, simple database seeding tool for Rust
<br/>

cder (_see-der_) is a database seeding tool to help you import fixure data in your local environment.

Generating seeds programmatically is a easy task, but maintaining them is not.
Everytime when your schema is changed, your seed can be broken.
It costs your team extra efforts to keep them updated.

#### with cder you can:
- maintain your data in a readable format, separated from the seeding program
- handle reference integrities on-the-fly, using **embedded tags**
- reuse existing struct and insert function, with only a little glue code is needed

cder has no mechanism for database interaction, so it can works with any type of ORM or database wrapper (e.g. sqlx) your application have.

This embedded-tag mechanism is inspired by [fixtures](https://github.com/rails/rails/blob/c9a0f1ab9616ca8e94f03327259ab61d22f04b51/activerecord/lib/active_record/fixtures.rb) that Ruby on Rails provides for test data generation.

## Installation

```toml
# Cargo.toml
[dependencies]
cder = "0.2"
```
## Usage

### Quick start

Suppose you have users table as seeding target:

```sql
CREATE TABLE
  users (
    `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
  )
```

In your application you also have:

- a struct of type `<T>` (usually a model, built upon a underlying table)
- database insertion method that returns id of the new record: `Fn(T) -> Result<i64>`

First, add DeserializeOwned trait on the struct.
(cder brings in *serde* as dependencies, so `derive(Deserialize)` macro can do the job)

```rust
use serde::Deserialize;

#[derive(Deserialize)] // add this derive macro
User {
  name: String,
  email: String,
}

impl User {
  // can be sync or async functions
  async fn insert(&self) -> Result<(i64)> {
    //
    // inserts a corresponding record into table, and returns its id when succeeded
    //
  }
}
```

Your User seed is defined by two separate files, data and glue code.

Now create a seed data file 'fixtures/users.yml'

```yaml
# fixtures/users.yml

User1:
  name: Alice
  email: 'alice@example.com'
User2:
  name: Bob
  email: 'bob@example.com'
```

Now you can insert above two users into your database:

```rust
use cder::DatabaseSeeder;

async fn populate_seeds() -> Result<()> {
    let mut seeder = DatabaseSeeder::new()

    seeder
        .populate_async("fixtures/users.yml", |input| {
            async move { User::insert(&input).await }
        })
        .await?;

    Ok(())
}
```

Et voila! You will get the records `Alice` and `Bob` populated in your database.

#### Working with non-async functions
If your function is non-async (normal) function, use `Seeder::populate` instead of `Seeder::populate_async`.

```rust
use cder::DatabaseSeeder;

fn main() -> Result<()> {
    let mut seeder = DatabaseSeeder::new();

    seeder
        .populate("fixures/users.yml", |input| {
            // this block can contain any non-async functions
            // but it has to return Result<i64> in the end
            diesel::insert_into(users)
                .values((name.eq(input.name), email.eq(input.email)))
                .returning(id)
                .get_result(conn)
                .map(|value| value.into())
        })
    
        Ok(())
}
```

### Constructing instances

If you want to take more granular control over the deserialized structs before inserting, use StructLoader instead.

```rust
use cder::{ Dict, StructLoader };

fn construct_users() -> Result<()> {
    // provide your fixture filename followed by its directory
    let mut loader = StructLoader::<User>::new("users.yml", "fixtures");

    // deserializes User struct from the given fixture
    // the argument is related to name resolution (described later)
    loader.load(&Dict::<String>::new())?;

    let customer = loader.get("User1")?;
    assert_eq!(customer.name, "Alice");
    assert_eq!(customer.email, "alice@example.com");

    let customer = loader.get("User2")?;
    assert_eq!(customer.name, "Bob");
    assert_eq!(customer.email, "bob@example.com");

    ok(())
}
```

### Defining values on-the-go
cder replaces certain tags with values based on a couple of rules.
This 'pre-processing' runs just before deserialization, so that you can define *dynamic* values that can vary depending on your local environments.

Currently following two cases are covered:

#### 1. Defining relations (foreign keys)

Let's say you have two records to be inserted in `companies` table.
`companies.id`s are unknown, as they are given by the local database on insert.

```yaml
# fixtures/companies.yml

Company1:
  name: MassiveSoft
Company2:
  name: BuggyTech
```

Now you have user records that reference to these companies:

```yaml
# fixtures/users.yml

User1:
  name: Alice
  company_id: 1 // this might be wrong
```

You might end up with failing building User1, as Company1 is not guaranteed to have id=1 (especially if you already have operated on the companies table).
For this, use `${{ REF(label) }}` tag in place of undecided values.

```yaml
User1:
  name: Alice
  company_id: ${{ REF(Company1) }}
```

Now, how does Seeder know id of Compnay1 record?
As described earlier, the block given to Seeder must return `Result<i64>`. Seeder stores the result value mapped against the record label, which will be re-used later to resolve the tag references.

```rust
use cder::DatabaseSeeder;

async fn populate_seeds() -> Result<()> {
    let mut seeder = DatabaseSeeder::new();
    // you can specify the base directory, relative to the project root
    seeder.set_dir("fixtures");

    // Seeder stores mapping of companies record label and its id
    seeder
        .populate_async("companies.yml", |input| {
            async move { Company::insert(&input).await }
        })
        .await?;
    // the mapping is used to resolve the reference tags
    seeder
        .populate_async("users.yml", |input| {
            async move { User::insert(&input).await }
        })
        .await?;

    Ok(())
}
```

A couple of watch-outs:
1. Insert a file that contains 'referenced' records first (`companies` in above examples) before 'referencing' records (`users`).
2. Currently Seeder resolve the tag when reading the source file. That means you cannot have references to the record within the same file.
If you want to reference a user record from another one, you could achieve this by splitting the yaml file in two.

#### 2. Environment vars
You can also refer to environment variables using `${{ ENV(var_name) }}` syntax.

```yaml
Dev:
  name: Developer
  email: ${{ ENV(DEVELOPER_EMAIL) }}
```

The email is replaced with `DEVELOPER_EMAIL` if that environment var is defined.

If you would prefer to use default value, use (shell-like) syntax:

```yaml
Dev:
  name: Developer
  email: ${{ ENV(DEVELOPER_EMAIL:-"developer@example.com") }}
```

Without specifying the defalut value, all the tags that points to undefined environment vars are simply replaced by empty string "".

### Data representation
cder deserializes yaml data based on [serde-yaml](https://github.com/dtolnay/serde-yaml), that supports powerful [serde serialization framework](https://serde.rs/). With serde, you can deserialize pretty much any struct. You can see a few [sample structs](tests/test_utils/types.rs) with various types of attributes and [the yaml files](tests/fixtures) that can be used as their seeds.

Below shows a few basics of required YAML format.
Check [serde-yaml's github page](https://github.com/dtolnay/serde-yaml) for further details.

#### Basics

```yaml
Label_1:
  name: Alice
  email: 'alice@example.com'
Label_2:
  name: Bob
  email: 'bob@example.com'
```

Notice that, cder requires to name each record with a label (*Label_x*).
Label can be anything (as long as it is a valid yaml key) but you might want to keep them unique to avoid accidental mis-references.

#### Enums and Complex types

Enums can be deserialized using YAML's `!tag`.
Suppose you have a struct CustomerProfile with enum `Contact`.

```rust
struct CustomerProfile {
  name: String,
  contact: Option<Contact>,
}

enum Contact {
  Email { email: String }
  Employee(usize),
  Unknown
}
```

You can generate customers with each type of contact as follows;

```yaml
Customer1:
  name: "Jane Doe"
  contact: !Email { email: "jane@example.com" }
Customer2:
  name: "Uncle Doe"
  contact: !Employee(10100)
Customer3:
  name: "John Doe"
  contact: !Unknown
```

### Not for production use
cder is designed to populate seeds in development (or possibly, test) environment. Production use is NOT recommended.

## License

The project is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as MIT, without any additional terms or conditions.

Bug reports and pull requests are welcome on GitHub at https://github.com/estie-inc/cder
