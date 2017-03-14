extern crate syn;
extern crate jsonapi;
extern crate inflector;

use super::quote::*;
use super::util;
use syn::DeriveInput;
use syn::Lit::*;
use syn::MetaItem::*;
use syn::NestedMetaItem::*;
use self::inflector::Inflector;

pub fn expand_json_api_fields(ast: &DeriveInput) -> Tokens {
    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().collect(),
        syn::Body::Enum(_) => panic!("#[derive(JsonApi)] can only be used with structs"),
    };

    let struct_rename_attr: Vec<_> = ast.attrs
        .iter()
        .filter_map(|a| match &a.value {
            &List(ref ident, ref values) if ident == "serde" => {
                match values.first() {
                    Some(&MetaItem(NameValue(ref i, Str(ref value, _)))) if i == "rename" => {
                        Some(value.to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect();

    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;
    let json_api_id = util::get_json_id(&fields);

    let attr_fields: Vec<_> = fields.iter().filter(|f| **f != json_api_id).collect();

    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let json_name = lower_case_name.to_string();
    // Shadows the json_name above - the variable above is only used for the unwrapping on the line below.
    let json_name = struct_rename_attr.first().unwrap_or(&json_name);

    // Used in the quasi-quotation below as `#generated_field_type_name`; append name + `Fields` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));

    let option_fields: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            quote!(#ident)
        })
        .collect();

    let sort_fields: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            quote!(#ident(SortOrder))
        })
        .collect();

    let sort_cases: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident_string = &f.ident.clone().expect("fail").to_string();
            let enum_value = Ident::new(format!("self::sort::{}", &ident_string));
            quote! {
            #ident_string => {
                sort_fields.push(#enum_value(sort_order))
            }
        }
        })
        .collect();

    let field_cases: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident_string = &f.ident.clone().expect("fail").to_string();
            let enum_value = Ident::new(format!("self::field::{}", &ident_string));
            quote! {
                #ident_string => {
                    foo.push(#enum_value)
                }
            }
        })
        .collect();

    quote! {
        pub mod #lower_case_name {
            use super::#name;
            use std::str::FromStr;
            use std::collections::HashSet;
            use std::collections::HashMap;
            use jsonapi::queryspec::*;
            use jsonapi::sort_order::SortOrder;

            #[derive(Debug, PartialEq, Eq)]
            #[allow(non_camel_case_types)]
            pub enum sort {
                #(#sort_fields),*
            }

            #[derive(Debug, PartialEq, Eq)]
            #[allow(non_camel_case_types)]
            pub enum field {
                //Expand field names into new struct
                #(#option_fields),*
            }

            pub struct #generated_params_type_name {
                pub fields: Vec<field>,
                pub sort_fields: Vec<sort>,
                pub query_params: HashMap<String, String>
            }

            impl FromStr for #generated_params_type_name {
                type Err = QueryStringParseError;
                fn from_str(query_string: &str) -> Result<#generated_params_type_name, QueryStringParseError> {
                    let mut foo:Vec<field> = Vec::new();
                    let mut sort_fields:Vec<sort> = Vec::new();
                    let mut query_params = HashMap::new();

                    for param in query_string.split('&') {
                        let mut split = param.split('=');
                        let key_value_pair = (split.next(), split.next());

                        if split.next() != None {
                            return Err(QueryStringParseError::InvalidParam(param.to_string()));
                        }

                        match key_value_pair {
                            (Some(""), None) | (None, None) => {},
                            (Some(key), None) => return Err(QueryStringParseError::InvalidValue(format!("Invalid param: {}", key))),
                            (None, Some(value)) => {
                                return Err(QueryStringParseError::InvalidValue(format!("Invalid param: {}", value)))
                            },
                            (Some(key), Some(value)) if key == "sort" => {
                                let fields = value.split(',').filter(|&f| !f.is_empty());
                                for mut field in fields {
                                    let sort_order = if field.starts_with('-') {
                                        field = field.trim_left_matches('-');
                                        SortOrder::Desc
                                    } else {
                                        SortOrder::Asc
                                    };

                                    match field {
                                        #(#sort_cases)*
                                        _ => return Err(QueryStringParseError::InvalidValue(format!("Invalid field: {}", field)))
                                    }
                                }
                            },
                            (Some(key), Some(value)) if key.starts_with("fields") => {
                                let mut model = key.trim_left_matches("fields");

                                if !model.starts_with('[') || !model.ends_with(']') {
                                    return Err(QueryStringParseError::InvalidKeyParam(model.to_string()));
                                }

                                model = model.trim_left_matches('[').trim_right_matches(']');

                                if model.is_empty() {
                                    return Err(QueryStringParseError::InvalidValue(format!("Value for {} is empty!", key)));
                                }

                                let fields:HashSet<_> = value.split(',').filter(|&f| !f.is_empty()).collect();

                                if fields.is_empty() {
                                    return Err(QueryStringParseError::InvalidValue(format!("Fields for {} are empty", model)))
                                }

                                match model.as_ref() {
                                    #json_name => {
                                        for field in fields {
                                            match field {
                                                #(#field_cases)*
                                                _ => return Err(QueryStringParseError::InvalidValue(format!("Invalid field: {}", field)))
                                            }
                                        }
                                    },

                                    // TODO: Implement non-existent field error here
                                    // TODO: Also implement relationship field filtering
                                    _ => return Err(QueryStringParseError::UnImplementedError)
                                }
                            },
                            (Some(key), Some(value)) => {
                                query_params.insert(key.to_string(), value.to_string());
                            }
                        }
                    }

                    Ok(#generated_params_type_name {
                       fields: foo,
                       sort_fields: sort_fields,
                       query_params: query_params
                    })
                }
            }

            impl ToParams for #name {
                type Params = #generated_params_type_name;
            }
        }
    }
}
