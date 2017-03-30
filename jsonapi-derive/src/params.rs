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
    let json_api_id_ty = &json_api_id.ty;

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

    let blaha: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident_string = &f.ident.clone().expect("fail").to_string();
            let enum_value = Ident::new(format!("self::field::{}", &ident_string));
            quote!(#enum_value)
        })
        .collect();

    let option_fields_len = option_fields.len();

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
                params.sort.fields.push(#enum_value(order))
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
                    params.filter.fields.push(#enum_value)
                }
            }
        })
        .collect();

    quote! {
        pub mod #lower_case_name {
            use super::#name;
            use std::slice::Iter;
            use std::collections::HashMap;
            use jsonapi::try_from::TryFrom;
            use jsonapi::params::Params;
            use jsonapi::params::TypedParams;
            use jsonapi::sort_order::SortOrder;
            use jsonapi::params::JsonApiResource;
            use jsonapi::query_string::QueryString;
            use jsonapi::queryspec::QueryStringParseError;

            #[derive(Debug, PartialEq, Eq, Clone)]
            #[allow(non_camel_case_types)]
            pub enum sort {
                #(#sort_fields),*
            }

            #[derive(Debug, PartialEq, Eq, Clone, Default)]
            pub struct Sort {
                pub fields: Vec<sort>
            }

            #[derive(Debug, PartialEq, Eq, Clone)]
            #[allow(non_camel_case_types)]
            pub enum field {
                //Expand field names into new struct
                #(#option_fields),*
            }

            #[derive(Debug, PartialEq, Eq, Clone, Default)]
            pub struct Filter {
                pub fields: Vec<field>
            }

            impl field {
                pub fn iter() -> Iter<'static, field> {
                    static FIELDS: [field;  #option_fields_len] = [#(#blaha),*];
                    FIELDS.into_iter()
                }
            }

            #[derive(Debug, PartialEq, Eq, Clone, Default)]
            pub struct #generated_params_type_name {
                pub filter: Filter,
                pub sort: Sort,
                pub query_params: HashMap<String, String>
            }

            impl TypedParams for #generated_params_type_name {
                type SortField = sort;
                type FilterField = field;

                fn filter(&mut self) -> &mut Vec<field> {
                    &mut self.filter.fields
                }

                fn sort(&mut self) -> &mut Vec<sort> {
                    &mut self.sort.fields
                }

                fn query_params(&mut self) -> &mut HashMap<String, String> {
                    &mut self.query_params
                }
            }

            /// Parses the sort query parameter.
            impl<'a> TryFrom<(&'a str, SortOrder, #generated_params_type_name)> for #generated_params_type_name {
                type Err = QueryStringParseError;

                fn try_from(mut tuple: (&'a str, SortOrder, #generated_params_type_name)) -> Result<Self, Self::Err> {
                    //TODO: Add duplicate sort checks? (i.e sort=foo,foo,-foo)?

                    let (field, order, mut params) = tuple;
                    match field {
                        #(#sort_cases)*
                        _ => return Err(QueryStringParseError::InvalidValue(format!("Invalid field: {}", field)))
                    }

                    Ok(params)
                }
            }

            /// Parses the field query parameter(s).
            impl<'a> TryFrom<(&'a str, Vec<&'a str>, #generated_params_type_name)> for #generated_params_type_name {
                type Err = QueryStringParseError;

                fn try_from(mut tuple: (&'a str, Vec<&'a str>, #generated_params_type_name)) -> Result<Self, Self::Err> {
                    let (model, fields, mut params) = tuple;
                    match model {
                        #json_name => {
                            // If we already have the same field in the map, we consider this an error.
                            if !params.filter.fields.is_empty() {

                            }

                            for field in fields {
                                match field {
                                    #(#field_cases)*
                                    _ => return Err(QueryStringParseError::InvalidValue(format!("Invalid field: {}", field)))
                                }
                            }
                        },
                        // TODO: Implement parsing of relationships
                        _ => return Err(QueryStringParseError::UnImplementedError)
                    }

                    Ok(params)
                }
            }

            impl Params for #generated_params_type_name {}

            impl <'a> QueryString<'a> for #name {
                type Params = #generated_params_type_name;
                type SortField = sort;
                type FilterField = field;
            }

            impl JsonApiResource for #name {
                type JsonApiIdType = #json_api_id_ty;
                type Params = #generated_params_type_name;
                type SortField = sort;
                type FilterField = field;
            }
        }
    }
}
