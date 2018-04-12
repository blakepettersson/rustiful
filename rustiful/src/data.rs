use params::JsonApiParams;
use resource::JsonApiResource;
use std::marker::PhantomData;
use to_json::ToJson;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// The JSONAPI representation of a resource.
pub struct JsonApiData<T>
where
    T: ToJson,
    T::Attrs: Clone
{
    /// The id of the JSONAPI resource.
    pub id: Option<String>,
    #[serde(rename = "type")]
    #[serde(serialize_with = "::json::phantomdata::serialize")]
    #[serde(deserialize_with = "::json::phantomdata::deserialize")]
    /// The type name of the JSONAPI resource, equivalent to the resource name.
    _type: PhantomData<T>,
    /// The attribute type of the JSONAPI resource.
    pub attributes: T::Attrs
}

impl<T> JsonApiData<T>
where
    T: ToJson,
    T::Attrs: Clone
{
    pub fn new<Id: Into<String>>(id: Option<Id>, attrs: T::Attrs) -> JsonApiData<T> {
        JsonApiData {
            id: id.map(|i| i.into()),
            _type: PhantomData,
            attributes: attrs
        }
    }

    /// Check if there is an id present.
    pub fn has_id(&self) -> bool {
        self.id.is_some()
    }
}

impl<T> Clone for JsonApiData<T>
where
    T: ToJson,
    T::Attrs: Clone
{
    fn clone(&self) -> Self {
        JsonApiData {
            id: self.id.clone(),
            _type: self._type.clone(),
            attributes: self.attributes.clone()
        }
    }
}

/// Converts `(T, T::Params)` to `JsonApiData<T>` for any `T` that implements `ToJson`.
///
/// This implementation is used by `IntoJson` to convert a resource with its parameters to its
/// JSONAPI representation. Prefer the use of `IntoJson` instead of using this implementation
/// directly.
///
/// * `model` - The resource to convert to its JSONAPI representation.
/// * `params` - Filters out fields that should not be serialized when sending to the client.
///
/// # Example
///
/// Given a resource that implements `ToJson` (this is automatically implemented when
/// deriving `JsonApi`), such as the one below:
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// Then you can convert a `MyResource` list to a list of `JsonApiData<MyResource>`.
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # use rustiful::IntoJson;
/// # use rustiful::JsonApiData;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// # struct MyResource {
/// #     id: String,
/// #     foo: bool,
/// #     bar: String
/// # }
/// #
/// # fn main() {
/// let resource = MyResource {
///     id: "foo".to_string(),
///     foo: true,
///     bar: "abc".to_string()
/// };
/// let resource_list: JsonApiData<MyResource> = (resource, &Default::default()).into();
/// # }
/// ```
impl<'a, T> From<(T, &'a JsonApiParams<T::FilterField, T::SortField>)> for JsonApiData<T>
where
    T: ToJson + JsonApiResource,
    T::Attrs: From<(T, &'a JsonApiParams<T::FilterField, T::SortField>)>
{
    fn from((model, params): (T, &'a JsonApiParams<T::FilterField, T::SortField>)) -> Self {
        JsonApiData::new(Some(model.id()), T::Attrs::from((model, params)))
    }
}

/// Converts `Self` into `T`. See the implementations to see what the conversions are intended for.
pub trait IntoJson<T, F: Default, S: FromStr> {
    fn into_json<'a>(self, params: &'a JsonApiParams<F, S>) -> T;
}

/// Converts `T` into `JsonApiData<T>` for any `T` that implements `ToJson`.
///
/// `params` - Filters out fields that should not be serialized when sending to the client.
///
/// # Example
///
/// Given a resource that implements `ToJson` (this is automatically implemented when
/// deriving `JsonApi`), such as the one below:
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// Then you can convert a `MyResource` into a `JsonApiData<MyResource>`.
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # use rustiful::IntoJson;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// # struct MyResource {
/// #     id: String,
/// #     foo: bool,
/// #     bar: String
/// # }
/// #
/// # fn main() {
/// let resource = MyResource {
///     id: "foo".to_string(),
///     foo: true,
///     bar: "abc".to_string()
/// };
/// let resource_list = resource.into_json(&Default::default());
/// # }
/// ```
impl<T> IntoJson<JsonApiData<T>, T::FilterField, T::SortField> for T
where
    T: ToJson + JsonApiResource,
    T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>
{
    fn into_json<'a>(
        self,
        params: &'a JsonApiParams<T::FilterField, T::SortField>
    ) -> JsonApiData<T> {
        (self, params).into()
    }
}

/// Converts `Vec<T>` into `Vec<JsonApiData<T>>` for any `T` that implements `ToJson`.
///
/// `params` - Filters out fields that should not be serialized when sending to the client.
///
/// # Example
///
/// Given a resource that implements `ToJson` (this is automatically implemented when
/// deriving `JsonApi`), such as the one below:
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// Then you can convert a `MyResource` list into a list of `JsonApiData<MyResource::Attrs>`.
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # use rustiful::IntoJson;
/// # use rustiful::JsonApiData;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// # struct MyResource {
/// #     id: String,
/// #     foo: bool,
/// #     bar: String
/// # }
/// #
/// # fn main() {
/// let resource = MyResource {
///     id: "foo".to_string(),
///     foo: true,
///     bar: "abc".to_string()
/// };
/// let resource_list: Vec<JsonApiData<MyResource>> = vec![resource].into_json(&Default::default());
/// # }
/// ```
impl<T> IntoJson<Vec<JsonApiData<T>>, T::FilterField, T::SortField> for Vec<T>
where
    T: ToJson + JsonApiResource,
    T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>
{
    fn into_json<'a>(
        self,
        params: &'a JsonApiParams<T::FilterField, T::SortField>
    ) -> Vec<JsonApiData<T>> {
        self.into_iter().map(|i| (i, params).into()).collect()
    }
}
