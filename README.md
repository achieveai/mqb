# MQB

MQB is a Rust library for constructing MongoDB filters and updates in a type-safe manner.

## Usage

1. Add the `KeyPathable` derive macro to your struct:

```rust
use serde::{Serialize, Deserialize};
use mqb_core::KeyPathable;

#[derive(Serialize, Deserialize, KeyPathable)]
pub struct DataObject {
    #[serde(rename = "_id")]
    id: ObjectId,

    name: String,

    version_id: i32,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    last_updated: chrono::DateTime<chrono::Utc>,
}
```

2. Create type-safe filters and updates:

```rust
use mqb_core::{kp::*, FilterBuilder, UpdateBuilder};

let filter = FilterBuilder::new()
    .eq(DataObject::kp().id(), user_id)
    .try_build()?;

let update = UpdateBuilder::new()
    .set(DataObject::kp().name(), "New Name")
    .inc(DataObject::kp().version_id(), 1)
    .current_date(DateObject::kp().last_updated())
    .try_build()?;
```

3. Finally perform the query:

```rust
let _ = collection
    .update_one(filter, update)
    .await?;
```

## Additional Details

### Serde Integration

MQB respects all serde attributes:

- `rename_all` and `rename` attributes for field name transformations
```rust
DataObject::kp().id() // Will be rendered as "_id" in the query.
```
- Custom serializers for fields using `serialize_with`
```rust
FilterBuilder::new()
    .lt(DataObject::kp().last_updated(), chrono::Utc::now())
    .try_build()?;

// The above will be rendered as the following, since the field last_updated has it's serializer overriden in the struct definition.
{
    "last_updated": {
        "$lte": bson::serde_helpers::chrono_datetime_as_bson_datetime::serialize(value), // where value is the argument passed to the `lte` method.
    }
}
``` 

### Type Safety

MQB prevents the following:

- Incorrect operator usage (e.g., `$inc` on String fields)
```rust
let filter = FilterBuilder::new()
    .inc(DataObject::kp().name(), 1) // Compile error, since name is a String.
```

- Invalid field references  
```rust
let filter = FilterBuilder::new()
    .eq(DataObject::kp().unknown_field(), "value") // Compile error, since unknown_field doesn't exist.
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
