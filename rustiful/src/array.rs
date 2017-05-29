use data::JsonApiData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiArray<T> {
    pub data: Vec<JsonApiData<T>>
}
