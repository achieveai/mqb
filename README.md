# MQB (MongoDB Query Builder)

MQB is a Rust library for constructing type-checked MongoDB filters and updates. It provides compile-time validation for MongoDB queries, addressing a common challenge in the MongoDB Rust ecosystem where query construction typically lacks type safety.

## Features

- **Compile-time Type Safety**: Catch query errors before runtime
- **Type-checked Field References**: Use keypaths to safely reference struct fields
- **Comprehensive Operator Support**: Type-safe implementations of MongoDB operators
- **Serde Integration**: Respects serde attributes and custom serializers
- **Intuitive API**: Builder pattern for constructing queries

## Quick Start

1. Add the `KeyPathable` derive macro to your struct:

```rust
use serde::{Serialize, Deserialize};
use mqb_core::KeyPathable;

#[derive(Serialize, Deserialize, KeyPathable)]
pub struct DataObject {
    #[serde(rename = "_id")]
    id: ObjectId,
    field0: String,
    version_id: i32,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    last_updated: chrono::DateTime<chrono::Utc>,
}
```

2. Create type-safe filters:

```rust
use mqb_core::{kp::*, FilterBuilder};

let filter = FilterBuilder::<DataObject>::new()
    .eq(
        DataObject::kp().field0(),
        "value"
    )
    .try_build()?;
```

3. Build type-safe updates:

```rust
use mqb_core::{kp::*, UpdateBuilder};

let update = UpdateBuilder::<DataObject>::new()
    .set(
        DataObject::kp().field0(),
        "new_value"
    )
    .inc(
        DataObject::kp().version_id(),
        1
    )
    .try_build()?;
```

4. Use with `Collection`:

```rust
collection
    .update_one(filter, update)
    .await?;
```

## Advanced Features

### Serde Integration

MQB respects all serde attributes and customizations:

- `rename_all` and `rename` attributes for field name transformations
```rust
DataObject::kp().id() // Will be rendered as "_id" in the query.
```
- Custom serializers for fields
```rust
FilterBuilder::<DataObject>::new()
    .lte(DataObject::kp().last_updated(), chrono::Utc::now())
    .try_build()?;

// The above will be rendered as:
{
    "last_updated": {
        "$lte": bson::serde_helpers::chrono_datetime_as_bson_datetime::serialize(value), // where value is the argument passed to the `lte` method.
    }
}
``` 
- Nested struct support
```rust
#[derive(Serialize, Deserialize, KeyPathable)]
struct Nested {
    field0: String,
}

// Allows for nested field access:

DataObject::kp().nested().field0() // Will be rendered as "nested.field0" in the query.
```

### Type Safety

MQB prevents the following:

- Incorrect operator usage (e.g., `$inc` on String fields)
```rust
let filter = FilterBuilder::<DataObject>::new()
    .inc(DataObject::kp().field0(), 1) // Compile error, since field0 is a String.
```
- Invalid field references  
```rust
let filter = FilterBuilder::<DataObject>::new()
    .eq(DataObject::kp().unknown_field(), "value") // Compile error, since unknown_field doesn't exist.
```


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
