use std::{collections::HashMap, marker::PhantomData};

use super::{
    KeyPathNodeLike, KeyPathNonInitialNodeLike, KeyPathable, SerializeFn,
};

pub struct HashMapKeyPathNode<Parent: KeyPathNodeLike, V, UnderlyingType> {
    key: &'static str,
    parent: Parent,
    serializer: SerializeFn<HashMap<String, V>>,
    marker: std::marker::PhantomData<UnderlyingType>,
}

impl<Parent: KeyPathNodeLike, V, UnderlyingType> Clone
    for HashMapKeyPathNode<Parent, V, UnderlyingType>
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            parent: self.parent.clone(),
            serializer: self.serializer,
            marker: PhantomData,
        }
    }
}

pub struct HMKeyKeyPathNode<Parent: KeyPathNodeLike, V> {
    key: String,
    parent: Parent,
    marker: PhantomData<V>,
}

impl<Parent: KeyPathNodeLike, V>
    HashMapKeyPathNode<Parent, V, HashMap<String, V>>
{
    pub fn key(self, key: String) -> HMKeyKeyPathNode<Self, V> {
        HMKeyKeyPathNode {
            key,
            parent: self.clone(),
            marker: PhantomData,
        }
    }
}

impl<V> KeyPathable for HashMap<String, V> {
    type KeyPathNode<Parent: KeyPathNodeLike, UnderlyingType> =
        HashMapKeyPathNode<Parent, V, UnderlyingType>;
}

impl<Parent: KeyPathNodeLike, V, UnderlyingType> KeyPathNodeLike
    for HashMapKeyPathNode<Parent, V, UnderlyingType>
{
    type Origin = Parent::Origin;

    type Current = HashMap<String, V>;

    fn render_path(&self) -> String {
        if Parent::IS_ROOT {
            self.key.to_owned()
        } else {
            format!("{}.{}", self.parent.render_path(), self.key)
        }
    }
}

impl<Parent: KeyPathNodeLike, V, UnderlyingType> KeyPathNonInitialNodeLike
    for HashMapKeyPathNode<Parent, V, UnderlyingType>
{
    type ParentNodeTy = Parent;
    type UnderlyingType = HashMap<String, V>;

    fn instance(
        key: &'static str,
        serializer: SerializeFn<Self::Current>,
        parent: Self::ParentNodeTy,
    ) -> Self {
        HashMapKeyPathNode {
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

impl<Parent: KeyPathNonInitialNodeLike, V> KeyPathNodeLike
    for HMKeyKeyPathNode<Parent, V>
{
    type Origin = Parent::Origin;
    type Current = V;

    fn render_path(&self) -> String {
        if Parent::IS_ROOT {
            self.key.to_owned()
        } else {
            format!(
                "{}.{}",
                KeyPathNonInitialNodeLike::render_path(&self.parent),
                self.key
            )
        }
    }
}

impl<Parent: KeyPathNonInitialNodeLike, V> KeyPathNonInitialNodeLike
    for HMKeyKeyPathNode<Parent, V>
where
    V: serde::Serialize,
{
    type ParentNodeTy = Parent;

    type UnderlyingType = V;

    fn instance(
        _key: &'static str,
        _serializer: SerializeFn<Self::Current>,
        _parent: Self::ParentNodeTy,
    ) -> Self {
        unimplemented!()
    }

    fn key(&self) -> String {
        self.key.to_owned()
    }

    fn parent(&self) -> &Self::ParentNodeTy {
        &self.parent
    }

    fn serializer(&self) -> SerializeFn<Self::Current> {
        bson::to_bson
    }
}

impl<Parent: KeyPathNonInitialNodeLike, V> Clone
    for HMKeyKeyPathNode<Parent, V>
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            parent: self.parent.clone(),
            marker: PhantomData,
        }
    }
}
