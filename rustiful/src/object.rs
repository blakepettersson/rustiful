use data::JsonApiData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiObject<T> {
    pub data: JsonApiData<T>
}
