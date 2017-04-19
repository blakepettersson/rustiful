use errors::QueryStringParseError;
use sort_order::SortOrder;
use std::collections::HashMap;
use std::str::FromStr;
use try_from::TryFrom;

pub trait JsonApiResource: Sized {
    type Params: Default + TypedParams<Self::SortField, Self::FilterField>;
    type SortField;
    type FilterField;
    type JsonApiIdType: FromStr;

    fn resource_name() -> &'static str;

    fn from_str<'a>(query_string: &'a str) -> Result<Self::Params, QueryStringParseError>
        where
            Self::Params :
            TryFrom<(&'a str, SortOrder, Self::Params), Error = QueryStringParseError> +
            TryFrom<(&'a str, Vec<&'a str>, Self::Params), Error = QueryStringParseError>
    {
        let mut params: Self::Params = Default::default();

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
                    if !params.sort().is_empty() {
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

                        match Self::Params::try_from((field, sort_order, params)) {
                            Ok(value) => params = value,
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

                    match Self::Params::try_from((model, fields, params)) {
                        Ok(value) => params = value,
                        Err(err) => return Err(err),
                    }
                }
                (Some(key), Some(value)) => {
                    params.query_params().insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(params)
    }
}

pub trait TypedParams<SortField, FilterField> {
    fn sort(&mut self) -> &mut Vec<SortField>;
    fn filter(&mut self) -> &mut Vec<FilterField>;
    fn query_params(&mut self) -> &mut HashMap<String, String>;
}
