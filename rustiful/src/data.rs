use params::JsonApiResource;
use to_json::ToJson;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiData<T> {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub lower_case_type: String,
    pub attributes: T,
}

impl<T> JsonApiData<T> {
    pub fn new<I: Into<String>>(id: Option<I>,
                                lower_case_type: String,
                                attrs: T)
                                -> JsonApiData<T> {
        JsonApiData {
            id: id.map(|i| i.into()),
            lower_case_type: lower_case_type,
            attributes: attrs,
        }
    }
}


/// Converts a `(T, T::Params)` to a `JsonApiData<T>`.
impl<'a, T> From<(T, &'a <T as JsonApiResource>::Params)> for JsonApiData<T::Attrs>
    where T: ToJson + JsonApiResource,
          T::Attrs: From<(T, &'a T::Params)>
{
    fn from((model, params): (T, &'a T::Params)) -> Self {
        JsonApiData::new(Some(model.id()),
                         model.type_name(),
                         T::Attrs::from((model, params)))
    }
}

/// Converts a `T` to a `JsonApiData<T>`.
impl<'a, T> From<T> for JsonApiData<T::Attrs>
    where T: ToJson + JsonApiResource,
          T::Params: 'a,
          T::Attrs: for<'b> From<(T, &'b T::Params)>
{
    fn from(model: T) -> Self {
        let params: T::Params = Default::default();
        JsonApiData::new(Some(model.id()),
                         model.type_name(),
                         T::Attrs::from((model, &params)))
    }
}
