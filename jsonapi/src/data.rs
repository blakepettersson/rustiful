use id::JsonApiId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiData<T> {
    pub id: JsonApiId,
    #[serde(rename = "type")]
    pub lower_case_type: String,
    pub attributes: T
}

impl <T> JsonApiData<T> {
    pub fn new<I: Into<JsonApiId>>(id: I, lower_case_type: String, attrs: T) -> JsonApiData<T> {
        JsonApiData {
            id: id.into(),
            lower_case_type: lower_case_type,
            attributes: attrs
        }
    }
}