use errors::QueryStringParseError;
use std::fmt::Debug;
use std::str::FromStr;

/// A trait that defines how to convert to/from a JSONAPI representation of the implementing type.
///
/// This trait is automatically implemented for any type that derives the `JsonApi` attribute.
pub trait JsonApiResource: Sized {
    /// An alias for `JsonApiParams<Self::SortField, Self::FilterField>`
    type Params: FromStr<Err = QueryStringParseError>;
    /// This type is typically generated in rustiful-derive.
    type SortField;
    /// This type is typically generated in rustiful-derive.
    type FilterField;
    /// The type of a field named `id` or the type of a field that has the `#[JsonApiId]` attribute
    /// on the type deriving `JsonApi`.
    type JsonApiIdType: FromStr + Debug;
    /// This is typically the pluralized, lower-cased and dasherized name of the type deriving
    /// `JsonApi`.
    const RESOURCE_NAME: &'static str;
}
