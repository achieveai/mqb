use std::collections::HashMap;

use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use mqb_core::{kp::KeyPathableAsRoot, UpdateBuilder};
use mqb_macro::KeyPathable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, KeyPathable)]
#[serde(rename_all = "PascalCase")]
pub struct Person {
    #[serde(rename = "_id")]
    id: ObjectId,

    #[serde(with = "crate::object_id_as_hex_string")]
    profile_id: ObjectId,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub time: DateTime<Utc>,

    pub age: i32,

    pub attempts: HashMap<String, Vec<i32>>,
}

#[test]
fn basic_update_test() {
    let oid = ObjectId::new();
    let profile_id = ObjectId::new();

    let update = UpdateBuilder::<Person>::new()
        .set(Person::kp().id(), oid)
        .current_date(Person::kp().time())
        .set_on_insert(Person::kp().profile_id(), profile_id)
        .push(Person::kp().attempts().key("attempt_id_1".to_string()), 12)
        .try_build()
        .unwrap();

    let expected = doc! {
        "$set": {
            "_id": oid,
        },
        "$push": {
            "Attempts.attempt_id_1": {
                "$each": [12]
            }
        },
        "$setOnInsert": {
            "ProfileId": profile_id.to_hex()
        },
        "$currentDate": {
            "Time": true
        },
    };

    assert_eq!(update, expected);
}

#[test]
fn test_empty_update() {
    let update = UpdateBuilder::<Person>::new().try_build().unwrap();

    let expected = doc! {};

    assert_eq!(update, expected);
}

#[test]
fn test_add_to_set_operations() {
    let update = UpdateBuilder::<Person>::new()
        .add_to_set(Person::kp().attempts().key("test1".to_string()), 1)
        .add_to_set(Person::kp().attempts().key("test1".to_string()), 2)
        .add_to_set(Person::kp().attempts().key("test2".to_string()), 3)
        .try_build()
        .unwrap();

    let expected = doc! {
        "$addToSet": {
            "Attempts.test1": {
                "$each": [1, 2]
            },
            "Attempts.test2": {
                "$each": [3]
            }
        }
    };

    assert_eq!(update, expected);
}

#[test]
fn test_current_date_multiple_fields() {
    let update = UpdateBuilder::<Person>::new()
        .current_date(Person::kp().time())
        .set(Person::kp().age(), 25)
        .try_build()
        .unwrap();

    let expected = doc! {
        "$currentDate": {
            "Time": true
        },
        "$set": {
            "Age": 25
        }
    };

    assert_eq!(update, expected);
}

#[test]
fn test_set_on_insert_multiple() {
    let profile_id = ObjectId::new();
    let now = Utc::now();

    let update = UpdateBuilder::<Person>::new()
        .set_on_insert(Person::kp().profile_id(), profile_id)
        .set_on_insert(Person::kp().time(), now)
        .set_on_insert(Person::kp().age(), 18)
        .try_build()
        .unwrap();

    let expected = doc! {
        "$setOnInsert": {
            "ProfileId": profile_id.to_hex(),
            "Time": now,
            "Age": 18
        }
    };

    assert_eq!(update, expected);
}

#[test]
fn test_combined_array_operations() {
    let update = UpdateBuilder::<Person>::new()
        .push(Person::kp().attempts().key("scores".to_string()), 90)
        .add_to_set(
            Person::kp().attempts().key("unique_scores".to_string()),
            95,
        )
        .add_to_set(
            Person::kp().attempts().key("unique_scores".to_string()),
            105,
        ) // Duplicate should be ignored
        .push(Person::kp().attempts().key("scores".to_string()), 85)
        .try_build()
        .unwrap();

    let expected = doc! {
        "$push": {
            "Attempts.scores": {
                "$each": [90, 85]
            }
        },
        "$addToSet": {
            "Attempts.unique_scores": {
                "$each": [95, 105]
            }
        }
    };

    assert_eq!(update, expected);
}

#[test]
fn test_all_operations_combined() {
    let profile_id = ObjectId::new();

    let update = UpdateBuilder::<Person>::new()
        // Set operations
        .set(Person::kp().age(), 30)
        .set_on_insert(Person::kp().profile_id(), profile_id)
        // Array operations
        .push(Person::kp().attempts().key("recent".to_string()), 100)
        .add_to_set(Person::kp().attempts().key("unique".to_string()), 105)
        // Increment operation
        .inc(Person::kp().age(), 1)
        // Date operation
        .current_date(Person::kp().time())
        .try_build()
        .unwrap();

    let expected = doc! {
        "$set": {
            "Age": 30
        },
        "$setOnInsert": {
            "ProfileId": profile_id.to_hex()
        },
        "$push": {
            "Attempts.recent": {
                "$each": [100]
            }
        },
        "$addToSet": {
            "Attempts.unique": {
                "$each": [105]
            }
        },
        "$inc": {
            "Age": 1_i64
        },
        "$currentDate": {
            "Time": true
        }
    };

    assert_eq!(update, expected);
}
