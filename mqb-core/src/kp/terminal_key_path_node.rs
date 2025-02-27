use std::marker::PhantomData;

use super::{
    KeyPathNodeLike, KeyPathNonInitialNodeLike, KeyPathable, SerializeFn,
};

pub struct TerminalKeyPathNode<Parent, T, UnderlyingType = T>
where
    Parent: KeyPathNodeLike,
{
    key: &'static str,
    parent: Parent,
    serializer: SerializeFn<T>,
    marker: PhantomData<UnderlyingType>,
}

impl<Parent: KeyPathNodeLike, T, UnderlyingType> Clone
    for TerminalKeyPathNode<Parent, T, UnderlyingType>
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            parent: self.parent.clone(),
            serializer: self.serializer,
            marker: self.marker,
        }
    }
}

impl<Parent: KeyPathNodeLike, T, UnderlyingType> KeyPathNodeLike
    for TerminalKeyPathNode<Parent, T, UnderlyingType>
{
    type Origin = Parent::Origin;
    type Current = T;

    fn render_path(&self) -> String {
        if Parent::IS_ROOT {
            self.key.to_owned()
        } else {
            format!("{}.{}", self.parent.render_path(), self.key)
        }
    }
}

impl<Parent: KeyPathNodeLike, T, UnderlyingType> KeyPathNonInitialNodeLike
    for TerminalKeyPathNode<Parent, T, UnderlyingType>
{
    type ParentNodeTy = Parent;
    type UnderlyingType = UnderlyingType;

    fn instance(
        key: &'static str,
        serializer: SerializeFn<Self::Current>,
        parent: Self::ParentNodeTy,
    ) -> Self {
        TerminalKeyPathNode {
            key,
            parent,
            serializer,
            marker: PhantomData,
        }
    }

    fn key(&self) -> String {
        self.key.to_owned()
    }

    fn parent(&self) -> &Self::ParentNodeTy {
        &self.parent
    }

    fn serializer(&self) -> SerializeFn<Self::Current> {
        self.serializer
    }
}

macro_rules! impl_key_pathable {
    ($($t:ty),*) => {
        $(
            impl KeyPathable for $t {
                type KeyPathNode<Parent: KeyPathNodeLike, UnderlyingType> = TerminalKeyPathNode<Parent, $t, UnderlyingType>;
            }
        )*
    };
}

impl_key_pathable!(String, i32, i64, f32, f64, bool, bson::oid::ObjectId);

#[cfg(feature = "chrono")]
impl_key_pathable!(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "uuid")]
impl_key_pathable!(uuid::Uuid);

impl<T> KeyPathable for Option<T> {
    type KeyPathNode<Parent: KeyPathNodeLike, UnderlyingType> =
        TerminalKeyPathNode<Parent, Option<T>, UnderlyingType>;
}

impl<T> KeyPathable for Vec<T> {
    type KeyPathNode<Parent: KeyPathNodeLike, UnderlyingType> =
        TerminalKeyPathNode<Parent, Vec<T>, UnderlyingType>;
}
