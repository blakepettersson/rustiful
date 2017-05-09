use errors::QueryStringParseError;
use sort_order::SortOrder;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use try_from::TryFrom;

#[derive(Debug, PartialEq, Eq, Clone)]
/// A type-safe container for incoming query parameters specific to JSONAPI.
pub struct JsonApiParams<F, S> {
    pub sort: Sort<S>,
    pub fieldset: FieldSet<F>,
    pub query_params: HashMap<String, String>,
}

impl<F, S> JsonApiParams<F, S> {
    fn new(fieldset: Vec<F>,
           sort_params: Vec<S>,
           query_params: HashMap<String, String>)
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
        let query_params: HashMap<String, String> = Default::default();
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

    fn from_str<'a>
        (query_string: &'a str)
         -> Result<JsonApiParams<Self::FilterField, Self::SortField>, QueryStringParseError>
        where Self::SortField: TryFrom<(&'a str, SortOrder), Error = QueryStringParseError>,
              Self::FilterField: TryFrom<(&'a str, Vec<&'a str>), Error = QueryStringParseError>
    {
        let mut params: JsonApiParams<Self::FilterField, Self::SortField> = Default::default();

        for param in query_string.split('&') {
            let mut split = param.split('=');
            let key_value_pair = (split.next(), split.next());

            if split.next() != None {
                return Err(QueryStringParseError::InvalidParam(param.to_string()));
            }

            match key_value_pair {
                (Some(""), None) | (None, None) => {}
                (Some(key), None) => {
                    return Err(QueryStringParseError::InvalidValue(format!("Invalid param: {}",
                                                                           key)))
                }
                (None, Some(value)) => {
                    return Err(QueryStringParseError::InvalidValue(format!("Invalid param: {}",
                                                                           value)))
                }
                (Some(key), Some(value)) if key == "sort" => {
                    if !params.sort.fields.is_empty() {
                        //TODO: Add duplicate key error variant
                        return Err(QueryStringParseError::InvalidValue(format!("Duplicate sort \
                                                                                key: {}",
                                                                               value)));
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
                            Ok(value) => params.sort.fields.push(value),
                            Err(err) => return Err(err),
                        }
                    }
                }
                (Some(key), Some(value)) if key.starts_with("fields") => {
                    let mut model = key.trim_left_matches("fields");

                    if !model.starts_with('[') || !model.ends_with(']') {
                        return Err(QueryStringParseError::InvalidKeyParam(model.to_string()));
                    }

                    model = model.trim_left_matches('[').trim_right_matches(']');

                    if model.is_empty() {
                        return Err(QueryStringParseError::InvalidValue(format!("Value for {} \
                                                                                is empty!",
                                                                               key)));
                    }

                    // This can introduce duplicates, but we don't really care. If there are
                    // duplicates it won't have any adverse effects - the field will still be
                    // visible.
                    let fields: Vec<_> = value.split(',').filter(|&f| !f.is_empty()).collect();

                    if fields.is_empty() {
                        return Err(QueryStringParseError::InvalidValue(format!("Fields for {} \
                                                                                are empty",
                                                                               model)));
                    }

                    match Self::FilterField::try_from((model, fields)) {
                        Ok(value) => params.fieldset.fields.push(value),
                        Err(err) => return Err(err),
                    }
                }
                (Some(key), Some(value)) => {
                    params.query_params.insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(params)
    }
}
