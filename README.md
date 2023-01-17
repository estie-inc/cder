[![cder](https://github.com/estie-inc/cder/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/estie-inc/cder/actions/workflows/ci.yml)

# cder


#### a lightweight, simple database seeding tool for Rust
<br/>

cder (_see-der_) is a database seeding tool to help you import
data in your local environment.

Creating, and maintaining seeds are always a headache.
Each time your schema is changed, your seed data or seeding program can be broken.

cder provides you a yaml-based simple seeding mechanism that allows you to:
- define data in a readable, easy-to-maintain format
- construct any data types to any Rust struct on your codebase
- reuse your existing insert function, no additional code required

With **embedded tags**, you can also define relations between records
(without even knowing their primary keys), as well as generate a
personally-customizable attributes using environment variables.

This embedded-tag mechanism is inspired by [fixtures](https://github.com/rails/rails/blob/c9a0f1ab9616ca8e94f03327259ab61d22f04b51/activerecord/lib/active_record/fixtures.rb) that Ruby on Rails provides for test data generation.

## Installation

```toml
# Cargo.toml
[dependencies]
cder = "0.1"
```
## Usage

### Quick start

#### constructing objects

Suppose you have User struct;
```rust
User {
  name: String,
  email: String,
}
```

First, create a fixture and save it as (let's assume) 'fixture/users.yml'
```yaml
User1:
  name: Alice
  email: 'alice@example.com'
User2:
  name: Bob
  email: 'bob@example.com'
```

Now you can build a Rust object as follows:

```rust
use cder::{ Dict, StructLoader };

fn construct_users() -> result<()> {
    // provide your fixture filename followed by its directory
    let mut loader = StructLoader::<User>::new("users.yml", "fixtures");
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

#### populating data

With DatabaseSeeder, you can also persist records as follows.

Suppose you have users table as following:
```rust
CREATE TABLE
  users (
    `id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL,
    `email` VARCHAR(255) NOT NULL,
  )
```

You need to have a struct (usually a domain model that is built upon the underlying table), with database insertion method (Fn(T) -> Result<i64>, or its async equivalent)
The struct need to have DeserializeOwned trait (don't worry, if you use serde, Deserialize macro can do the job)

```rust
use serde::Deserialize;

#[derive(Deserialize, Clone)]
User {
  name: String,
  email: String,
}

impl User {
  async fn insert(&self) -> Result<(i64)> {
    // inserts a corresponding record into table, and returns its id when succeeded
  }
}
```

First, create a fixture and save it as 'fixture/users.yml'

```yaml
User1:
  name: Alice
  email: 'alice@example.com'
User2:
  name: Bob
  email: 'bob@example.com'
```

Now you can insert above two users into your database:

```rust
use cder::{ Dict, StructLoader };

async fn populate_users() -> result<()> {
    // provide base directory (as Option)
    let mut seeder = DatabaseSeeder::new("fixtures");
    let ids = seeder
        .populate_async("users.yml", |input: User| {
            Box::pin(async move { User::insert().await })
        })
        .await?;

    ok(())
}
```

Et voila! You will get the data populated in your database.

### Data representation

#### Basics
The basic structure of YAML file should be as follows:

```yaml
Label_1:
  name: Alice
  email: 'alice@example.com'
Label_2:
  name: Bob
  email: 'bob@example.com'
```

Label_x is the name of each record (whatever string is fine, but should be unique), and values below defines the attributes.

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

#### Further customization
cder deserializes yaml data based on [serde-yaml](https://github.com/dtolnay/serde-yaml), that supports powerful [serde serialization framework](https://serde.rs/). With serde, you can deserialize pretty much any struct. You can see a few [sample structs](tests/test_utils/types.rs) with various types of attributes and [the yaml files](tests/fixtures) that can be used as their seeds.

Check [serde-yaml's github page](https://github.com/dtolnay/serde-yaml) for further details.

### Defining relations
* TODO

### Environment vars
* TODO

#### Default value
* TODO

## License

The project is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, shall be licensed as MIT, without any additional terms or conditions.

Bug reports and pull requests are welcome on GitHub at https://github.com/estie-inc/cder
