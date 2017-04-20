extern crate syn;
extern crate rustiful;
extern crate inflector;

use self::inflector::Inflector;
use super::quote::*;
use super::util;
use syn::Attribute;
use syn::DeriveInput;
use syn::Lit::*;
use syn::MetaItem::*;
use syn::NestedMetaItem::*;

pub fn expand_json_api_fields(ast: &DeriveInput) -> Tokens {
    let (id, fields) = util::get_attrs_and_id(&ast.body);

    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;
    let lower_case_name = name.to_string().to_snake_case();
    let json_api_id_ty = &id.ty;

    let attr_fields: Vec<_> = fields.iter()
        .map(|f| f.ident.clone().expect("#[derive(JsonApi)] is not supported for tuple structs"))
        .collect();

    let json_name = get_json_name(&lower_case_name, &ast.attrs);
    let lower_cased_ident = Ident::new(lower_case_name);
    let pluralized_name = json_name.to_plural().to_kebab_case();

    // Used in the quasi-quotation below as `#params_name`;
    // append name + `Params` to the new struct name
    let params_name = Ident::new(format!("__{}{}", name, "Params"));

    let option_fields: Vec<_> = attr_fields.iter().map(|f| quote!(#f)).collect();
    let option_fields_len = option_fields.len();

    let filter_fields: Vec<_> = attr_fields.iter().map(|f| quote!(self::field::#f)).collect();
    let filter_cases: Vec<_> = attr_fields.iter()
        .map(|f| to_match_arm(f, &quote!(params.filter.fields), &quote!(self::field::#f)))
        .collect();

    let sort_fields: Vec<_> = attr_fields.iter().map(|f| quote!(#f(SortOrder))).collect();
    let sort_cases: Vec<_> = attr_fields.iter()
        .map(|f| to_match_arm(f, &quote!(params.sort.fields), &quote!(self::sort::#f(order))))
        .collect();

    quote! {
        pub mod #lower_cased_ident {
            use super::#name;
            use std::slice::Iter;
            use std::collections::HashMap;
            use rustiful::TryFrom;
            use rustiful::SortOrder;
            use rustiful::TypedParams;
            use rustiful::JsonApiResource;
            use rustiful::QueryStringParseError;

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
                    static FIELDS: [field;  #option_fields_len] = [#(#filter_fields),*];
                    FIELDS.into_iter()
                }
            }

            #[derive(Debug, PartialEq, Eq, Clone, Default)]
            pub struct #params_name {
                pub filter: Filter,
                pub sort: Sort,
                pub query_params: HashMap<String, String>
            }

            impl TypedParams<sort, field> for #params_name {
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
            impl<'a> TryFrom<(&'a str, SortOrder, #params_name)> for #params_name {
                type Error = QueryStringParseError;

                fn try_from(tuple: (&'a str, SortOrder, #params_name)) -> Result<Self, Self::Error> {
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
            impl<'a> TryFrom<(&'a str, Vec<&'a str>, #params_name)> for #params_name {
                type Error = QueryStringParseError;

                fn try_from(tuple: (&'a str, Vec<&'a str>, #params_name)) -> Result<Self, Self::Error> {
                    let (model, fields, mut params) = tuple;
                    match model {
                        #json_name => {
                            // If we already have the same field in the map, we consider this an error.
                            if !params.filter.fields.is_empty() {

                            }

                            for field in fields {
                                match field {
                                    #(#filter_cases)*
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

            impl JsonApiResource for #name {
                type JsonApiIdType = #json_api_id_ty;
                type Params = #params_name;
                type SortField = sort;
                type FilterField = field;

                fn resource_name() -> &'static str {
                    #pluralized_name.as_ref()
                }
            }
        }
    }
}

fn to_match_arm(ident: &syn::Ident, vec_ident: &Tokens, enum_value: &Tokens) -> Tokens {
    let ident_string = ident.to_string();
    quote!(#ident_string => { #vec_ident.push(#enum_value) })
}

fn get_json_name(name: &str, attrs: &[Attribute]) -> String {
    let serde_struct_rename_attr: Vec<_> = attrs.into_iter()
        .filter_map(|a| match a.value {
            List(ref ident, ref values) if ident == "serde" => {
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

    serde_struct_rename_attr.first().map(|s| s.to_string()).unwrap_or_else(|| name.to_string())
}