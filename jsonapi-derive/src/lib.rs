#![crate_type = "proc-macro"]
#![recursion_limit = "512"]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate quote;

extern crate syn;
extern crate jsonapi;
extern crate proc_macro;

use quote::*;
use syn::Ty;
use syn::Lit::*;
use syn::MetaItem::*;
use syn::NestedMetaItem::*;
use proc_macro::TokenStream;

#[proc_macro_derive(JsonApi, attributes(JsonApiId))]
pub fn json_api(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = expand_json_api_fields(&ast);

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}


fn is_option_ty(ty: &Ty) -> bool {
    let option_ident = Ident::new("Option");
    match *ty {
        Ty::Path(_, ref path) => {
            path.segments.first()
                .map(|s| s.ident == option_ident)
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn inner_of_option_ty(ty: &Ty) -> Option<&Ty> {
    use syn::PathParameters::AngleBracketed;

    if !is_option_ty(ty) {
        return None;
    }

    match *ty {
        Ty::Path(_, syn::Path { ref segments, .. }) =>
            match segments[0].parameters {
                AngleBracketed(ref data) => data.types.first(),
                _ => None,
            },
        _ => None,
    }
}

fn expand_json_api_fields(ast: &syn::DeriveInput) -> quote::Tokens {
    let fields:Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().collect(),
        syn::Body::Enum(_) => panic!("#[derive(JsonApi)] can only be used with structs"),
    };

    let struct_rename_attr:Vec<_> = ast
        .attrs
        .iter()
        .filter_map(|a| {
            match &a.value {
                &List(ref ident, ref values) if ident == "serde" => {
                    match values.first() {
                        Some(&MetaItem(NameValue(ref i, Str(ref value, _)))) if i == "rename" => {
                            Some(value.to_string())
                        },
                        _ => None
                    }
                },
                _ => None
            }
        })
        .collect();

    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let id = fields.iter().find(|f| f.ident.iter().any(|i| i.to_string() == "id"));
    let json_api_id_attrs:Vec<_> = fields.iter().filter(|f| f.attrs.iter().any(|a| a.name() == "JsonApiId")).collect();

    if json_api_id_attrs.len() > 1 {
        panic!("Invalid: Only one field is allowed to have the JsonApiId attribute!")
    }

    let json_api_attr_id = json_api_id_attrs.first();

    if id != None && json_api_attr_id != None {
        panic!("You can only use a JsonApiId attribute or have an id field, not both at the same time.")
    }

    let json_api_id = id.or(json_api_attr_id.cloned()).expect("No JsonApiId attribute defined! (or no field named id)");
    let json_api_id_ty = &json_api_id.ty;

    let attr_fields:Vec<_> = fields.iter().filter(|f| f != &json_api_id).collect();

    let lower_case_name = Ident::new(name.to_string().to_lowercase());
    let json_name = lower_case_name.to_string();
    // Shadows the json_name above - the variable above is only used for the unwrapping on the line below.
    let json_name = struct_rename_attr.first().unwrap_or(&json_name);

    // Used in the quasi-quotation below as `#generated_field_type_name`; append name + `Fields` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));

    let generated_jsonapi_attrs = Ident::new(format!("__{}{}", name, "JsonApiAttrs"));
    let generated_jsonapi_resource = Ident::new(format!("__{}{}", name, "JsonApiResource"));

    let option_fields:Vec<_> = attr_fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident)
    }).collect();

    let sort_fields:Vec<_> = attr_fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident(SortOrder))
    }).collect();

    let jsonapi_attrs:Vec<_> = attr_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let option_ty = inner_of_option_ty(ty).unwrap_or(ty);

        quote!(#ident: #option_ty)
    }).collect();

    let sort_cases:Vec<_> = attr_fields.iter().map(|f| {
        let ident_string = &f.ident.clone().expect("fail").to_string();
        let enum_value = Ident::new(format!("self::sort::{}", &ident_string));
        quote! {
            #ident_string => {
                sort_fields.push(#enum_value(sort_order))
            }
        }
    }).collect();

    let field_cases:Vec<_> = attr_fields.iter().map(|f| {
        let ident_string = &f.ident.clone().expect("fail").to_string();
        let enum_value = Ident::new(format!("self::field::{}", &ident_string));
        quote! {
            #ident_string => {
                foo.push(#enum_value)
            }
        }

    }).collect();

    let bla = quote! {
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

            pub struct #generated_jsonapi_attrs {
                #(#jsonapi_attrs),*
            }

            pub struct #generated_jsonapi_resource {
                id: #json_api_id_ty,
                attributes: #generated_jsonapi_attrs
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
    };

    bla
}