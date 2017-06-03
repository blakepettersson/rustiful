extern crate url;

use self::url::form_urlencoded;
use errors::QueryStringParseError;
use sort_order::SortOrder;
use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::collections::hash_map::Entry::Vacant;
use std::fmt::Debug;
use std::str::FromStr;
use try_from::TryFrom;

#[derive(Debug, PartialEq, Eq, Clone)]
/// A type-safe container for all incoming query parameters in a request.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// use std::collections::HashMap;
///
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// // This enum will typically be generated by rustiful-derive
/// enum GeneratedSortParam {
///     foo(rustiful::SortOrder),
///     bar(rustiful::SortOrder)
/// }
///
/// // This enum will typically be generated by rustiful-derive
/// #[derive(Debug, PartialEq, Eq, Clone)]
/// enum GeneratedFieldSetParam {
///     foo,
///     bar,
/// }
///
/// let fields = vec![GeneratedFieldSetParam::foo, GeneratedFieldSetParam::bar];
/// let sort = vec![GeneratedSortParam::foo(rustiful::SortOrder::Asc)];
/// let query_params = HashMap::new();
///
/// let params = rustiful::JsonApiParams::new(fields.clone(), sort.clone(), query_params.clone());
///
/// assert_eq!(fields, params.fieldset.fields);
/// assert_eq!(sort, params.sort.fields);
/// assert_eq!(query_params, params.query_params);
/// ```
pub struct JsonApiParams<F, S> {
    /// A type-safe container for the "sort" query parameter in JSONAPI.
    ///
    /// The type parameter `<S>` will be an enum type that is generated using the `JsonApi`
    /// attribute in rustiful-derive.
    pub sort: Sort<S>,
    /// A type-safe container for the "fields" query parameter in JSONAPI.
    ///
    /// The type parameter `<F>` will be an enum type that is generated using the `JsonApi`
    /// attribute in rustiful-derive.
    pub fieldset: FieldSet<F>,
    /// A hashmap representing all query parameters that is not "sort" nor "fields[*]".
    pub query_params: HashMap<String, Vec<String>>,
}

impl<F, S> JsonApiParams<F, S> {
    pub fn new(fieldset: Vec<F>,
               sort_params: Vec<S>,
               query_params: HashMap<String, Vec<String>>)
               -> JsonApiParams<F, S> {
        JsonApiParams {
            sort: Sort { fields: sort_params },
            fieldset: FieldSet { fields: fieldset },
            query_params: query_params,
        }
    }
}

impl<F, S> Default for JsonApiParams<F, S> {
    fn default() -> Self {
        let query_params: HashMap<String, Vec<String>> = Default::default();
        JsonApiParams::new(vec![], vec![], query_params)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// A type-safe container for the "sort" query parameter in JSONAPI.
///
/// The type parameter `<S>` will be an enum type that is generated using the `JsonApi` attribute in
/// rustiful-derive.
pub struct Sort<S> {
    pub fields: Vec<S>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// A type-safe container for the "fields" query parameter in JSONAPI.
///
/// The type parameter `<F>` will be an enum type that is generated using the `JsonApi` attribute in
/// rustiful-derive.
pub struct FieldSet<F> {
    pub fields: Vec<F>,
}

/// This trait is implemented for any type that derives the `JsonApi` attribute.
pub trait JsonApiResource: Sized {
    /// An alias for `JsonApiParams<Self::SortField, Self::FilterField>`
    type Params: Default;
    /// This type is generated in rustiful-derive.
    type SortField;
    /// This type is generated in rustiful-derive.
    type FilterField;
    /// The type of a field named `id` or the type of a field that has the `#[JsonApiId]` attribute
    /// on the type deriving `JsonApi`.
    type JsonApiIdType: FromStr + Debug;
    /// This is the pluralized lower-cased name of the type deriving `JsonApi`.
    fn resource_name() -> &'static str;

    /// Converts a query string to a type-safe representation.
    ///
    /// This function converts a query string to a
    /// `JsonApiParams<Self::SortField, Self::FilterField>` type. This is aliased as `Self::Params`.
    ///
    /// Returns `Ok(Self::Params)` if there are no errors when attempting to parse the query string.
    ///
    /// # Errors
    /// * If any field name in the `fields` query parameter doesn't match with any of the field
    /// names in the type deriving this trait (or rather if the string doesn't match with a string
    /// present in the `TryFrom` impl of the generated `field` enum.)
    /// * If any field name in the `sort` query parameter doesn't match with any of the field
    /// names in the type deriving this trait (or rather if the string doesn't match with a string
    /// present in the `TryFrom` impl of the generated `sort` enum.)
    ///
    ///
    /// # Example
    ///
    /// ```
    /// extern crate rustiful;
    ///
    /// use rustiful::TryFrom;
    /// use rustiful::SortOrder;
    /// use rustiful::JsonApiParams;
    /// use rustiful::JsonApiResource;
    /// use rustiful::QueryStringParseError;
    /// use std::collections::HashMap;
    ///
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// #[allow(non_camel_case_types)]
    /// // This enum will typically be generated by rustiful-derive
    /// enum sort {
    ///     foo(SortOrder),
    ///     bar(SortOrder)
    /// }
    ///
    /// // This impl will typically be generated by rustiful-derive
    /// impl<'a> TryFrom<(&'a str, SortOrder)> for sort {
    ///     type Error = QueryStringParseError;
    ///
    ///     fn try_from((field, order): (&'a str, SortOrder)) -> Result<Self, Self::Error> {
    ///         match field {
    ///             "foo" => return Ok(sort::foo(order)),
    ///             "bar" => return Ok(sort::bar(order)),
    ///             _ => return Err(QueryStringParseError::InvalidValue(field.to_string()))
    ///         }
    ///     }
    /// }
    ///
    /// // This enum will typically be generated by rustiful-derive
    /// #[derive(Debug, PartialEq, Eq, Clone)]
    /// #[allow(non_camel_case_types)]
    /// enum field {
    ///     foo,
    ///     bar,
    /// }
    ///
    /// // This impl will typically be generated by rustiful-derive
    /// impl<'a> TryFrom<(&'a str, Vec<&'a str>)> for field {
    ///     type Error = QueryStringParseError;
    ///
    ///     fn try_from((model, fields): (&'a str, Vec<&'a str>)) -> Result<Self, Self::Error> {
    ///         match model {
    ///             "my-resources" => {
    ///                 for field in fields {
    ///                     match field {
    ///                         "foo" => return Ok(field::foo),
    ///                         "bar" => return Ok(field::bar),
    ///                         _ => {
    ///                             let field_val = field.to_string();
    ///                             return Err(QueryStringParseError::InvalidValue(field_val))
    ///                         }
    ///                     }
    ///                 }
    ///             },
    ///                  // TODO: Implement parsing of relationships
    ///             _ => return Err(QueryStringParseError::UnImplementedError)
    ///         }
    ///
    ///         // TODO: Implement parsing of relationships
    ///         return Err(QueryStringParseError::UnImplementedError)
    ///     }
    /// }
    ///
    /// // This impl will typically be generated by rustiful-derive
    /// impl JsonApiResource for MyResource {
    ///     type Params = JsonApiParams<Self::SortField, Self::FilterField>;
    ///     type SortField = sort;
    ///     type FilterField = field;
    ///     type JsonApiIdType = String;
    ///
    ///     fn resource_name() -> &'static str {
    ///         "my-resources"
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let fields = vec![field::bar];
    ///     let sort = vec![sort::foo(SortOrder::Desc)];
    ///     let mut query_params = HashMap::new();
    ///     query_params.insert("other".to_string(), vec!["test".to_string(), "abc".to_string()]);
    ///
    ///     let expected = JsonApiParams::new(fields, sort, query_params);
    ///     let query_string = "sort=-foo&fields[my-resources]=bar&other=test&other=abc";
    ///     let params = MyResource::from_str(query_string);
    ///     assert_eq!(expected, params.unwrap());
    /// }
    /// ```
    fn from_str<'a>
        (query_string: &'a str)
         -> Result<JsonApiParams<Self::FilterField, Self::SortField>, QueryStringParseError>
        where Self::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              Self::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>),
                                                 Error = QueryStringParseError>
    {
        let mut params: JsonApiParams<Self::FilterField, Self::SortField> = Default::default();
        let decoded = form_urlencoded::parse(query_string.as_bytes()).into_owned();

        for (key, value) in decoded {
            if &key == "sort" {
                if !params.sort.fields.is_empty() {
                    return Err(QueryStringParseError::DuplicateSortKey(value));
                }

                let fields = value.split(',').filter(|&f| !f.is_empty());
                for mut field in fields {
                    let sort_order = if field.starts_with('-') {
                        field = field.trim_left_matches('-');
                        SortOrder::Desc
                    } else {
                        SortOrder::Asc
                    };

                    match Self::SortField::try_from((field, sort_order)) {
                        Ok(result) => params.sort.fields.push(result),
                        Err(err) => return Err(err),
                    }
                }

            } else if key.starts_with("fields") {
                let mut model = key.trim_left_matches("fields");

                if !model.starts_with('[') || !model.ends_with(']') {
                    return Err(QueryStringParseError::InvalidKeyParam(model.to_string()));
                }

                model = model.trim_left_matches('[').trim_right_matches(']');

                if model.is_empty() {
                    return Err(QueryStringParseError::EmptyFieldsetKey(key.to_string()));
                }

                // This can introduce duplicates, but we don't really care. If there are
                // duplicates it won't have any adverse effects - the field will still be
                // visible.
                let fields: Vec<_> = value.split(',').filter(|&f| !f.is_empty()).collect();

                if fields.is_empty() {
                    return Err(QueryStringParseError::EmptyFieldsetValue(model.to_string()));
                }

                match Self::FilterField::try_from((model, fields)) {
                    Ok(result) => params.fieldset.fields.push(result),
                    Err(err) => return Err(err),
                }

            } else {
                match params.query_params.entry(key) {
                    // Already a Vec here, push onto it
                    Occupied(entry) => {
                        entry.into_mut().push(value);
                    }
                    // No value, create a one-element Vec.
                    Vacant(entry) => {
                        entry.insert(vec![value]);
                    }
                };
            }
        }

        Ok(params)
    }
}
