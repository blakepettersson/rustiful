use id::JsonApiId;
use queryspec::ToJson;
use params::TypedParams;
use params::JsonApiResource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiData<T> {
    pub id: JsonApiId,
    #[serde(rename = "type")]
    pub lower_case_type: String,
    pub attributes: T,
}

impl<T> JsonApiData<T> {
    pub fn new(id: JsonApiId, lower_case_type: String, attrs: T) -> JsonApiData<T> {
        JsonApiData {
            id: id,
            lower_case_type: lower_case_type,
            attributes: attrs,
        }
    }
}

impl<'a, T> From<(T, &'a <T as JsonApiResource>::Params)>
    for JsonApiData<T::Json>
    where T: ToJson,
          T: JsonApiResource,
          T::Json: From<(T, &'a <T as JsonApiResource>::Params)>
{
    fn from(tuple: (T, &'a <T as JsonApiResource>::Params)) -> Self {
        let (model, params) = tuple;
        JsonApiData::new(model.id(), model.type_name(), T::Json::from((model, params)))
    }
}
