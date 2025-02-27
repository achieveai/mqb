mod terminal_key_path_node;
pub use terminal_key_path_node::*;

mod hashmap_key_path_node;
pub use hashmap_key_path_node::*;

pub type SerializeFn<T> = fn(&T) -> bson::ser::Result<bson::Bson>;

pub trait KeyPathable {
    type KeyPathNode<Parent: KeyPathNodeLike, UnderlyingType>: KeyPathNonInitialNodeLike<ParentNodeTy = Parent>;
}

pub trait KeyPathableAsRoot: KeyPathable + Sized {
    type RootKeyPathNode: KeyPathInitialNodeLike<Origin = Self>;

    fn kp() -> Self::RootKeyPathNode;
}

pub trait KeyPathNodeLike: Clone {
    type Origin;
    type Current;

    const IS_ROOT: bool = false;

    fn render_path(&self) -> String;
}

pub trait KeyPathNonInitialNodeLike: KeyPathNodeLike {
    type ParentNodeTy: KeyPathNodeLike;
    type UnderlyingType;

    fn instance(
        key: &'static str,
        serializer: SerializeFn<Self::Current>,
        parent: Self::ParentNodeTy,
    ) -> Self;

    fn key(&self) -> String;
    fn parent(&self) -> &Self::ParentNodeTy;
    fn serializer(&self) -> SerializeFn<Self::Current>;

    fn render_path(&self) -> String {
        if Self::ParentNodeTy::IS_ROOT {
            self.key().to_string()
        } else {
            format!("{}.{}", self.parent().render_path(), self.key())
        }
    }
}

pub trait KeyPathInitialNodeLike: KeyPathNodeLike {}

pub fn render<KP: KeyPathNodeLike>(node: &KP) -> String {
    node.render_path()
}

pub struct UnknownUnderlyingType;
