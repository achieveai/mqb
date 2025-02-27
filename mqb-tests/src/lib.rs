#[cfg(test)]
mod filter_tests;

#[cfg(test)]
mod update_tests;

pub mod object_id_as_hex_string {
    use bson::oid::ObjectId;
    use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

    /// Deserializes an ObjectId from a hex string.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object_id_str: &str = Deserialize::deserialize(deserializer)?;
        ObjectId::parse_str(object_id_str).map_err(D::Error::custom)
    }

    /// Serializes an ObjectId as a hex string.
    pub fn serialize<S: Serializer>(
        val: &ObjectId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        val.to_hex().serialize(serializer)
    }
}
