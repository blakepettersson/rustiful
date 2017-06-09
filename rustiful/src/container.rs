#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Contains JSONAPI data.
///
/// `data` should be either `JsonApiData<T::Attrs>` or `Vec<JsonApiData<T:Attrs>>`, where `T` is an
/// implementation of `ToJson`.
pub struct JsonApiContainer<T> {
    pub data: T
}
