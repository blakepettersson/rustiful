#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiArray<T> {
    pub data: Vec<T>
}