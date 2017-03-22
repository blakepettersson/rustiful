
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiId(JsonApiIdType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonApiIdType {
    Str(String),
    Int32(i32)
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