use params::JsonApiParams;
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
    pub fn new<Id: Into<String>, Type: Into<String>>(id: Option<Id>,
                                                     lower_case_type: Type,
                                                     attrs: T)
                                                     -> JsonApiData<T> {
        JsonApiData {
            id: id.map(|i| i.into()),
            lower_case_type: lower_case_type.into(),
            attributes: attrs,
        }
    }

    /// Check if there is an id present.
    pub fn has_id(&self) -> bool {
        self.id.is_some()
    }
}


/// Converts a `(T, T::Params)` to a `JsonApiData<T>`.
impl<'a, T> From<(T, &'a JsonApiParams<T::FilterField, T::SortField>)> for JsonApiData<T::Attrs>
    where T: ToJson + JsonApiResource,
          T::Attrs: From<(T, &'a JsonApiParams<T::FilterField, T::SortField>)>
{
    fn from((model, params): (T, &'a JsonApiParams<T::FilterField, T::SortField>)) -> Self {
        JsonApiData::new(Some(model.id()),
                         model.type_name(),
                         T::Attrs::from((model, params)))
    }
}

pub trait IntoJson<T, F, S> {
    fn into_json<'a>(self, params: &'a JsonApiParams<F, S>) -> T;
}

impl<T> IntoJson<JsonApiData<T::Attrs>, T::FilterField, T::SortField> for T
    where T: ToJson + JsonApiResource,
          T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>
{
    fn into_json<'a>(self,
                     params: &'a JsonApiParams<T::FilterField, T::SortField>)
                     -> JsonApiData<T::Attrs> {
        (self, params).into()
    }
}

impl<T> IntoJson<Vec<JsonApiData<T::Attrs>>, T::FilterField, T::SortField> for Vec<T>
    where T: ToJson + JsonApiResource,
          T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>
{
    fn into_json<'a>(self,
                     params: &'a JsonApiParams<T::FilterField, T::SortField>)
                     -> Vec<JsonApiData<T::Attrs>> {
        self.into_iter().map(|i| (i, params).into()).collect()
    }
}
