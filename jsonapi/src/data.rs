use id::JsonApiId;
use queryspec::ToJson;
use params::JsonApiResource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiData<T> {
    pub id: Option<JsonApiId>,
    #[serde(rename = "type")]
    pub lower_case_type: String,
    pub attributes: T,
}

impl<T> JsonApiData<T> {
    pub fn new(id: Option<JsonApiId>, lower_case_type: String, attrs: T) -> JsonApiData<T> {
        JsonApiData {
            id: id,
            lower_case_type: lower_case_type,
            attributes: attrs,
        }
    }
}

impl<'a, T> From<(T, &'a <T as JsonApiResource>::Params)> for JsonApiData<T::Attrs>
    where T: ToJson + JsonApiResource,
          T::Attrs: From<(T, &'a T::Params)>
{
    // In this case we know that there's going to be a Some(JsonApiId).
    fn from(tuple: (T, &'a T::Params)) -> Self {
        let (model, params) = tuple;
        JsonApiData::new(Some(model.id()),
                         model.type_name(),
                         T::Attrs::from((model, params)))
    }
}

impl<'a, T> From<T> for JsonApiData<T::Attrs>
    where T: ToJson + JsonApiResource,
          T::Params : 'a,
          T::Attrs: for <'b> From<(T, &'b T::Params)>
{
    // In this case we know that there's going to be a Some(JsonApiId).
    fn from(model: T) -> Self {
        let params: T::Params = Default::default();
        JsonApiData::new(Some(model.id()), model.type_name(), T::Attrs::from((model, &params)))
    }
}
