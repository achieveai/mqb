use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Utc};
use mqb_core::kp::KeyPathableAsRoot;
use mqb_macro::KeyPathable;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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

    pub address: Address,

    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, KeyPathable)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    city: String,
    street: String,
    kind: AddressKind,
}

#[derive(Serialize_repr, Deserialize_repr, KeyPathable, PartialEq)]
#[repr(u8)]
pub enum AddressKind {
    Home = 0,
    Work = 1,
}

#[test]
fn eq_test() {
    let oid = ObjectId::new();

    let filter = mqb_core::FilterBuilder::new()
        .gt(Person::kp().address().city(), "New York".to_string())
        .eq(Person::kp().profile_id(), oid)
        .eq(Person::kp().address().city(), "New York".to_string())
        .eq(Person::kp().address().kind(), AddressKind::Home)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Address.City": {
                "$gt": "New York",
                "$eq": "New York"
            },

            "ProfileId": {
                "$eq": oid.to_hex()
            },

            "Address.Kind": {
                "$eq": AddressKind::Home as i32
            }
        }
    );
}

#[test]
fn neq_test() {
    let oid = ObjectId::new();

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .ne(Person::kp().id(), oid)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "_id": {
                "$ne": oid
            }
        }
    );
}

#[test]
fn custom_field_serialization_strategy_oid_as_hex() {
    let oid = ObjectId::new();

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .eq(Person::kp().profile_id(), oid)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "ProfileId": {
                "$eq": oid.to_hex()
            }
        }
    );
}

#[test]
fn custom_field_serialization_strategy_chronodt_as_bsondt() {
    let ts = Utc::now();

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .eq(Person::kp().time(), ts)
        .try_build()
        .unwrap();
    let bson_dt = bson::DateTime::from_chrono(ts);

    assert_eq!(
        filter,
        doc! {
            "Time": {
                "$eq": bson_dt
            }
        }
    );
}

#[test]
fn gt_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .gt(Person::kp().age(), 18)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Age": {
                "$gt": 18
            }
        }
    );
}

#[test]
fn lt_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .lt(Person::kp().age(), 18)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Age": {
                "$lt": 18
            }
        }
    );
}

#[test]
fn filter_fields_test() {
    let chrono_dt = Utc::now() - chrono::TimeDelta::days(10);
    let bson_dt: bson::DateTime = chrono_dt.into();

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .lt(Person::kp().age(), 18)
        .gt(Person::kp().time(), chrono_dt)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Age": {
                "$lt": 18
            },
            "Time": {
                "$gt": bson_dt
            }
        }
    );
}

#[test]
fn exists_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .exists::<true, _, _>(Person::kp().nickname())
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Nickname": {
                "$exists": true
            }
        }
    );
}

#[test]
fn exists_with_value_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .exists::<false, _, _>(Person::kp().nickname())
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Nickname": {
                "$exists": false
            }
        }
    );
}

#[test]
fn is_some_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .exists::<true, _, _>(Person::kp().nickname())
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Nickname": {
                "$exists": true
            }
        }
    );
}

#[test]
fn is_none_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .exists::<false, _, _>(Person::kp().nickname())
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Nickname": {
                "$exists": false
            }
        }
    );
}

#[test]
fn and_test() {
    let filter = mqb_core::FilterBuilder::<Person>::new()
        .gt(Person::kp().age(), 18)
        .lt(Person::kp().age(), 65)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Age": {
                "$gt": 18,
                "$lt": 65
            }
        }
    );
}

#[test]
fn complex_filter_test() {
    let oid = ObjectId::new();
    let ts = Utc::now();
    let bson_dt = bson::DateTime::from_chrono(ts);

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .ne(Person::kp().id(), oid)
        .gt(Person::kp().age(), 18)
        .lt(Person::kp().age(), 65)
        .gt(Person::kp().time(), ts)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "_id": {
                "$ne": oid
            },
            "Age": {
                "$gt": 18,
                "$lt": 65
            },
            "Time": {
                "$gt": bson_dt
            }
        }
    );
}

#[test]
fn multiple_and_conditions_test() {
    let oid = ObjectId::new();
    let ts = Utc::now();
    let bson_dt = bson::DateTime::from_chrono(ts);

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .gt(Person::kp().age(), 18)
        .lt(Person::kp().age(), 65)
        .ne(Person::kp().age(), 21)
        .lt(Person::kp().time(), ts)
        .eq(Person::kp().profile_id(), oid)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Age": {
                "$gt": 18,
                "$lt": 65,
                "$ne": 21
            },
            "Time": {
                "$lt": bson_dt
            },
            "ProfileId": {
                "$eq": oid.to_hex()
            }
        }
    );
}

#[test]
fn mixed_optional_and_required_fields_test() {
    let ts = Utc::now();
    let bson_dt = bson::DateTime::from_chrono(ts);

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .exists::<true, _, _>(Person::kp().nickname())
        .gt(Person::kp().age(), 21)
        .lt(Person::kp().age(), 30)
        .lt(Person::kp().time(), ts)
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "Nickname": {
                "$exists": true
            },
            "Age": {
                "$gt": 21,
                "$lt": 30
            },
            "Time": {
                "$lt": bson_dt
            }
        }
    );
}

#[test]
fn all_field_types_combined_test() {
    let profile_oid = ObjectId::new();
    let id_oid = ObjectId::new();
    let ts = Utc::now();
    let bson_dt = bson::DateTime::from_chrono(ts);

    let filter = mqb_core::FilterBuilder::<Person>::new()
        .ne(Person::kp().id(), id_oid)
        .eq(Person::kp().profile_id(), profile_oid)
        .gt(Person::kp().age(), 18)
        .lt(Person::kp().age(), 65)
        .ne(Person::kp().age(), 25)
        .gt(Person::kp().time(), ts)
        .exists::<true, _, _>(Person::kp().nickname())
        .try_build()
        .unwrap();

    assert_eq!(
        filter,
        doc! {
            "_id": {
                "$ne": id_oid
            },
            "ProfileId": {
                "$eq": profile_oid.to_hex()
            },
            "Age": {
                "$gt": 18,
                "$lt": 65,
                "$ne": 25
            },
            "Time": {
                "$gt": bson_dt
            },
            "Nickname": {
                "$exists": true
            }
        }
    );
}
