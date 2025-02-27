use std::marker::PhantomData;

use crate::kp::KeyPathNonInitialNodeLike;
use bson::doc;
use num_traits::PrimInt;
use serde::Serialize;

#[derive(Default)]
pub struct UpdateBuilder<T> {
    document: bson::Document,
    marker: PhantomData<T>,
    error: Option<bson::ser::Error>,
}

impl<T> UpdateBuilder<T> {
    pub fn new() -> Self {
        Self {
            document: bson::Document::new(),
            marker: PhantomData,
            error: None,
        }
    }

    pub fn try_build(self) -> Result<bson::Document, bson::ser::Error> {
        if let Some(error) = self.error {
            Err(error)
        } else {
            Ok(self.document)
        }
    }
}

impl<T> UpdateBuilder<T> {
    pub fn set<KP, V>(mut self, kp: KP, value: impl Into<V>) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        let serializer = kp.serializer();

        let bson = match serializer(&value.into()) {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        let path = crate::kp::render(&kp);

        if self.document.get_document("$set").is_err() {
            self.document.insert("$set", bson::Document::new());
        }

        let set = self.document.get_document_mut("$set").unwrap();
        set.insert(path, bson);

        self
    }

    pub fn set_on_insert<KP, V>(mut self, kp: KP, value: impl Into<V>) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        let serializer = kp.serializer();

        let bson = match serializer(&value.into()) {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        let path = crate::kp::render(&kp);

        if self.document.get_document("$setOnInsert").is_err() {
            self.document.insert("$setOnInsert", bson::Document::new());
        }

        let set_on_insert =
            self.document.get_document_mut("$setOnInsert").unwrap();
        set_on_insert.insert(path, bson);

        self
    }

    pub fn push<KP, V>(mut self, kp: KP, value: impl Into<V>) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = Vec<V>>,
        V: Serialize,
    {
        let path = crate::kp::render(&kp);
        let bson = match bson::to_bson(&value.into()) {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        if self.document.get_document("$push").is_err() {
            self.document.insert("$push", bson::Document::new());
        }

        let push = self.document.get_document_mut("$push").unwrap();
        if push.contains_key(&path) {
            let current_value = push.get_document_mut(&path).unwrap();
            let current_value_array =
                current_value.get_array_mut("$each").unwrap();
            current_value_array.push(bson);
        } else {
            push.insert(
                path,
                doc! {
                    "$each": [bson]
                },
            );
        }

        self
    }

    pub fn current_date<KP>(mut self, kp: KP) -> Self
    where
        KP: KeyPathNonInitialNodeLike<
            Origin = T,
            UnderlyingType = bson::DateTime,
        >,
    {
        let path = crate::kp::render(&kp);

        if self.document.get_document("$currentDate").is_err() {
            self.document.insert("$currentDate", bson::Document::new());
        }

        let current_date =
            self.document.get_document_mut("$currentDate").unwrap();
        current_date.insert(path, true);

        self
    }

    pub fn add_to_set<KP, V>(mut self, kp: KP, value: impl Into<V>) -> Self
    where
        KP: KeyPathNonInitialNodeLike<
            Origin = T,
            Current: IntoIterator<Item = V>,
        >,
        V: Serialize,
    {
        let path = crate::kp::render(&kp);
        let bson = match bson::to_bson(&value.into()) {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        if self.document.get_document("$addToSet").is_err() {
            self.document.insert("$addToSet", bson::Document::new());
        }

        let add_to_set = self.document.get_document_mut("$addToSet").unwrap();
        if add_to_set.contains_key(&path) {
            let current_value = add_to_set.get_document_mut(&path).unwrap();
            let current_value_array =
                current_value.get_array_mut("$each").unwrap();
            current_value_array.push(bson);
        } else {
            add_to_set.insert(
                path,
                doc! {
                    "$each": [bson]
                },
            );
        }

        self
    }

    pub fn inc<KP>(mut self, kp: KP, amount: i64) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current: PrimInt>,
    {
        let path = crate::kp::render(&kp);

        if self.document.get_document("$inc").is_err() {
            self.document.insert("$inc", bson::Document::new());
        }

        let inc = self.document.get_document_mut("$inc").unwrap();
        inc.insert(path, amount);

        self
    }
}
