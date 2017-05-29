extern crate syn;
extern crate rustiful;
extern crate inflector;

use self::inflector::Inflector;
use super::quote::*;
use syn::Attribute;
use syn::Lit::*;
use syn::MetaItem::*;
use syn::NestedMetaItem::*;
use util;
use util::JsonApiField;

pub fn expand_json_api_fields(name: &syn::Ident,
                              attrs: &[Attribute],
                              &(ref id, ref fields): &(JsonApiField, Vec<JsonApiField>))
                              -> Tokens {
    let lower_case_name = name.to_string().to_snake_case();
    let json_api_id_ty = &id.field.ty;

    let json_name = get_json_name(&lower_case_name, attrs);
    let lower_cased_ident = Ident::new(lower_case_name);
    let pluralized_name = json_name.to_plural().to_kebab_case();

    let mut option_fields: Vec<_> = Vec::with_capacity(fields.len());
    let option_fields_len = fields.len();

    let mut filter_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut filter_cases: Vec<_> = Vec::with_capacity(fields.len());
    let mut sort_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut sort_cases: Vec<_> = Vec::with_capacity(fields.len());

    for field in fields {
        let f = &field.ident;

        option_fields.push(quote!(#f));

        sort_fields.push(quote!(#f(SortOrder)));
        sort_cases.push(to_match_arm(&f, &quote!(self::sort::#f(order))));

        filter_fields.push(quote!(self::field::#f));
        filter_cases.push(to_match_arm(&f, &quote!(self::field::#f)));
    }

    let uuid = util::get_uuid_tokens();

    quote! {
        pub mod #lower_cased_ident {
            #uuid

            use super::#name;
            use std::slice::Iter;
            use rustiful::TryFrom;
            use rustiful::SortOrder;
            use rustiful::JsonApiParams;
            use rustiful::JsonApiResource;
            use rustiful::QueryStringParseError;

            #[derive(Debug, PartialEq, Eq, Clone)]
            #[allow(non_camel_case_types)]
            pub enum sort {
                #(#sort_fields),*
            }

            #[derive(Debug, PartialEq, Eq, Clone)]
            #[allow(non_camel_case_types)]
            pub enum field {
                //Expand field names into new struct
                #(#option_fields),*
            }

            impl field {
                pub fn iter() -> Iter<'static, field> {
                    static FIELDS: [field;  #option_fields_len] = [#(#filter_fields),*];
                    FIELDS.into_iter()
                }
            }

            impl<'a> TryFrom<(&'a str, SortOrder)> for sort {
                type Error = QueryStringParseError;

                fn try_from((field, order): (&'a str, SortOrder)) -> Result<Self, Self::Error> {
                    match field {
                        #(#sort_cases),*
                        _ => return Err(QueryStringParseError::InvalidValue(field.to_string()))
                    }
                }
            }

            impl<'a> TryFrom<(&'a str, Vec<&'a str>)> for field {
                type Error = QueryStringParseError;

                fn try_from((model, fields): (&'a str, Vec<&'a str>)) -> Result<Self, Self::Error> {
                    match model {
                        #json_name => {
                            for field in fields {
                                match field {
                                    #(#filter_cases),*
                                    _ => {
                                        let field_val = field.to_string();
                                        return Err(QueryStringParseError::InvalidValue(field_val))
                                    }
                                }
                            }
                        },
                        // TODO: Implement parsing of relationships
                        _ => return Err(QueryStringParseError::UnImplementedError)
                    }

                    // TODO: Implement parsing of relationships
                    return Err(QueryStringParseError::UnImplementedError)
                }
            }

            impl JsonApiResource for #name {
                type JsonApiIdType = #json_api_id_ty;
                type Params = JsonApiParams<field, sort>;
                type SortField = sort;
                type FilterField = field;

                fn resource_name() -> &'static str {
                    #pluralized_name.as_ref()
                }
            }
        }
    }
}

fn to_match_arm(ident: &syn::Ident, enum_value: &Tokens) -> Tokens {
    let ident_string = ident.to_string();
    quote!(#ident_string => { return Ok(#enum_value) })
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
