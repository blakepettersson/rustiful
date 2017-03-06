#![crate_type = "proc-macro"]
#![recursion_limit = "256"]

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

#[proc_macro_derive(JsonApi)]
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

    let idents:Vec<_> = fields.iter().filter_map(|f| f.ident.clone()).collect();

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

    let lower_case_name = Ident::new(name.to_string().to_lowercase());
    let json_name = lower_case_name.to_string();
    // Shadows the json_name above - the variable above is only used for the unwrapping on the line below.
    let json_name = struct_rename_attr.first().unwrap_or(&json_name);

    // Used in the quasi-quotation below as `#generated_field_type_name`; append name + `Fields` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));

    // Used in the quasi-quotation below as `#generated_field_type_name`; append name + `Fields` to the new struct name
    let generated_field_type_name = Ident::new(format!("__{}{}", name, "Fields"));

    let vars:Vec<_> = idents.iter().map(|i| {
        let field_var = Ident::new(format!("{}{}", "field_", i.to_string()));
        quote! {
            let mut #field_var = false;
        }
    }).collect();

    let option_fields:Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(pub #ident : bool)
    }).collect();

    let sort_fields:Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote!(#ident(SortOrder))
    }).collect();

    let sort_cases:Vec<_> = fields.iter().map(|f| {
        let ident_string = &f.ident.clone().expect("fail").to_string();
        let enum_value = Ident::new(format!("self::sort::{}", &ident_string));
        quote! {
            #ident_string => {
                sort_fields.push(#enum_value(sort_order))
            }
        }
    }).collect();

    let field_cases:Vec<_> = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident_string = &f.ident.clone().expect("fail").to_string();
        let field_var = Ident::new(format!("{}{}", "field_", ident_string));
        quote!(#ident_string => #field_var = true,)
    }).collect();

    let field_setters:Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        let ident_string = &f.ident.clone().expect("fail").to_string();
        let field_var = Ident::new(format!("{}{}", "field_", ident_string));
        quote!(#ident: #field_var,)
    }).collect();

    let bla = quote! {
        pub mod #lower_case_name {
            use super::#name;
            use std::str::FromStr;
            use std::collections::HashSet;
            use jsonapi::queryspec::*;
            use jsonapi::sort_order::SortOrder;

            #[derive(Debug, PartialEq, Eq)]
            #[allow(non_camel_case_types)]
            pub enum sort {
                #(#sort_fields),*
            }

            pub struct #generated_field_type_name {
                //Expand field names into new struct
                #(#option_fields),*
            }

            pub struct #generated_params_type_name {
                pub fields: #generated_field_type_name,
                pub sort_fields: Vec<sort>
            }

            impl FromStr for #generated_params_type_name {
                type Err = QueryStringParseError;
                fn from_str(query_string: &str) -> Result<#generated_params_type_name, QueryStringParseError> {
                    #(#vars)*

                    let mut sort_fields:Vec<sort> = Vec::new();

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
                            (Some(_), Some(_)) => {
                                return Err(QueryStringParseError::UnImplementedError)
                            }
                        }
                    }

                    Ok(#generated_params_type_name {
                       fields: #generated_field_type_name {
                            #(#field_setters)*
                       },
                       sort_fields: sort_fields
                    })
                }
            }

            impl ToParams for #name {
                type Params = #generated_params_type_name;
            }

            impl ToSortFields for #name {
                type SortField = sort;
            }
        }
    };

    //println!("{}", bla);
    bla
}