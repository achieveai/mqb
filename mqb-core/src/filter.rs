use crate::kp::KeyPathNonInitialNodeLike;

#[derive(Default)]
pub struct FilterBuilder<T> {
    document: bson::Document,
    error: Option<bson::ser::Error>,
    marker: std::marker::PhantomData<T>,
}

impl<T> FilterBuilder<T> {
    pub fn new() -> Self {
        Self {
            document: bson::Document::new(),
            error: None,
            marker: std::marker::PhantomData,
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

impl<T> FilterBuilder<T> {
    pub fn eq<KP, V: PartialEq>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$eq")
    }

    pub fn ne<KP, V: PartialEq>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$ne")
    }

    pub fn gt<KP, V: PartialOrd>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$gt")
    }

    pub fn gte<KP, V: PartialOrd>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$gte")
    }

    pub fn lt<KP, V: PartialOrd>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$lt")
    }

    pub fn lte<KP, V: PartialOrd>(self, kp: KP, value: V) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        self.op(kp, value, "$lte")
    }

    pub fn exists<const EXISTS: bool, KP, V>(mut self, kp: KP) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = Option<V>>,
    {
        let keypath = crate::kp::render(&kp);

        if self.document.get_document(&keypath).is_err() {
            self.document.insert(&keypath, bson::Document::new());
        }

        let exists = self.document.get_document_mut(&keypath).unwrap();
        exists.insert("$exists", EXISTS);

        self
    }

    pub fn r#in<KP, V>(mut self, kp: KP, values: Vec<V>) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        let serializer = kp.serializer();

        let bson_values = match values
            .into_iter()
            .map(|value| serializer(&value))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        let keypath = crate::kp::render(&kp);

        if self.document.get_document(&keypath).is_err() {
            self.document.insert(&keypath, bson::Document::new());
        }

        let in_ = self.document.get_document_mut(&keypath).unwrap();
        in_.insert("$in", bson_values);

        self
    }
}

impl<T> FilterBuilder<T> {
    fn op<KP, V>(mut self, kp: KP, value: V, op: &'static str) -> Self
    where
        KP: KeyPathNonInitialNodeLike<Origin = T, Current = V>,
    {
        let serializer = kp.serializer();

        let bson = match serializer(&value) {
            Ok(bson) => bson,
            Err(e) => {
                self.error = Some(e);
                return self;
            }
        };

        let keypath = crate::kp::render(&kp);

        if self.document.get_document(&keypath).is_err() {
            self.document.insert(&keypath, bson::Document::new());
        }

        let eq = self.document.get_document_mut(&keypath).unwrap();
        eq.insert(op, bson);

        self
    }
}
