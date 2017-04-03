#[cfg(feature = "uuid")]
extern crate uuid;

#[cfg(feature = "uuid")]
use self::uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiId(JsonApiIdType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonApiIdType {
    Str(String),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    #[cfg(feature = "uuid")]
    Uuid(Uuid),
}

impl From<String> for JsonApiId {
    fn from(value: String) -> Self {
        JsonApiId(JsonApiIdType::Str(value))
    }
}

impl From<i32> for JsonApiId {
    fn from(value: i32) -> Self {
        JsonApiId(JsonApiIdType::Int32(value))
    }
}

impl From<i64> for JsonApiId {
    fn from(value: i64) -> Self {
        JsonApiId(JsonApiIdType::Int64(value))
    }
}

impl From<u32> for JsonApiId {
    fn from(value: u32) -> Self {
        JsonApiId(JsonApiIdType::UInt32(value))
    }
}

impl From<u64> for JsonApiId {
    fn from(value: u64) -> Self {
        JsonApiId(JsonApiIdType::UInt64(value))
    }
}

#[cfg(feature = "uuid")]
impl From<Uuid> for JsonApiId {
    fn from(value: Uuid) -> Self {
        JsonApiId(JsonApiIdType::Uuid(value))
    }
}
