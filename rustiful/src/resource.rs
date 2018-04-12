use errors::QueryStringParseError;
use std::fmt::Debug;
use std::str::FromStr;

extern crate serde_qs as qs;

/// A trait that defines how to convert to/from a JSONAPI representation of the implementing type.
///
/// This trait is automatically implemented for any type that derives the `JsonApi` attribute.
pub trait JsonApiResource: Sized {
    /// An alias for `JsonApiParams<Self::SortField, Self::FilterField>`
    type Params: FromStr<Err = qs::Error>;
    /// This type is typically generated in rustiful-derive.
    type SortField: FromStr;
    /// This type is typically generated in rustiful-derive.
    type FilterField: Default;
    /// The type of a field named `id` or the type of a field that has the `#[JsonApiId]` attribute
    /// on the type deriving `JsonApi`.
    type JsonApiIdType: FromStr + Debug;
    /// This is typically the pluralized, lower-cased and dasherized name of the type deriving
    /// `JsonApi`.
    const RESOURCE_NAME: &'static str;
}
