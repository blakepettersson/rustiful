extern crate inflector;
extern crate syn;

use self::inflector::Inflector;
use super::quote::*;
use syn::Attribute;
use syn::MetaItem::*;
use syn::NestedMetaItem::MetaItem;
use syn::Lit::Str;
use util;
use util::JsonApiField;

pub fn expand_json_api_fields(
    name: &syn::Ident,
    attrs: &[Attribute],
    &(ref id, ref fields): &(JsonApiField, Vec<JsonApiField>)
) -> Tokens {
    let json_api_id_ty = &id.field.ty;

    let lower_case_name = name.to_string().to_snake_case();
    let pluralized_name = lower_case_name.to_plural().to_kebab_case();
    let json_name = get_json_name( attrs).map(|a| {
        quote!(
                #a
                #[serde(deserialize_with = "_rustiful::json::comma_separated::deserialize")]
        )
    }).unwrap_or(quote!(
                #[serde(deserialize_with = "_rustiful::json::comma_separated::deserialize")]
        ));

    let lower_cased_ident = Ident::new(lower_case_name);

    let mut option_fields: Vec<_> = Vec::with_capacity(fields.len());

    let mut filter_cases: Vec<_> = Vec::with_capacity(fields.len());
    let mut sort_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut sort_cases: Vec<_> = Vec::with_capacity(fields.len());

    for field in fields {
        let f = &field.ident;

        option_fields.push(quote!(#f));

        sort_fields.push(quote!(#f(SortOrder)));
        sort_cases.push(to_match_arm(&f, &quote!(self::sort::#f(order))));

        //filter_fields.push(quote!(self::field::#f));
        filter_cases.push(to_match_arm(&f, &quote!(self::field::#f)));
    }

    let uuid = util::get_uuid_tokens();

    quote! {
        pub mod #lower_cased_ident {
            #uuid

            extern crate serde;
            extern crate rustiful as _rustiful;

            use super::#name;
            use std::slice::Iter;
            use std::str::FromStr;
            use self::serde::Deserialize;
            use self::_rustiful::SortOrder;
            use self::_rustiful::JsonApiParams;
            use self::_rustiful::JsonApiResource;
            use self::_rustiful::QueryStringParseError;

            #[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
            #[allow(non_camel_case_types)]
            pub struct wat {
                #[serde(deserialize_with = "_rustiful::json::comma_separated::deserialize")]
                pub #lower_cased_ident: Vec<sort>
            }

            #[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
            #[allow(non_camel_case_types)]
            pub struct wat2 {
                #json_name
                pub #lower_cased_ident: Vec<field>
            }

            #[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum sort {
                #(#sort_fields),*
            }

            impl FromStr for sort {
                type Err = QueryStringParseError;

                fn from_str(field: &str) -> Result<Self, Self::Err> {
                    let order = SortOrder::from(field);
                    match field.trim_left_matches("-") {
                        #(#sort_cases),*
                        _ => return Err(QueryStringParseError::InvalidSortValue(field.to_string()))
                    }
                }
            }

            #[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
            #[allow(non_camel_case_types)]
            pub enum field {
                //Expand field names into new struct
                #(#option_fields),*
            }

            impl FromStr for field {
                type Err = QueryStringParseError;

                fn from_str(field: &str) -> Result<Self, Self::Err> {
                    match field {
                        #(#filter_cases),*
                        _ => {
                            let field_val = field.to_string();
                            let e = QueryStringParseError::InvalidFieldValue(field_val);
                            return Err(e)
                        }
                    }
                }
            }

            impl JsonApiResource for #name {
                type JsonApiIdType = #json_api_id_ty;
                type Params = JsonApiParams<wat2, sort>;
                type SortField = sort;
                type FilterField = wat2;
                const RESOURCE_NAME: &'static str = #pluralized_name;
            }
        }
    }
}

fn to_match_arm(ident: &syn::Ident, enum_value: &Tokens) -> Tokens {
    let ident_string = ident.to_string();
    quote!(#ident_string => { return Ok(#enum_value) })
}

fn get_json_name<'a>(attrs: &'a [Attribute]) -> Option<&'a Attribute> {
    let serde_struct_rename_attr = attrs
        .into_iter()
        .find(|a| match a.value {
            List(ref ident, ref values) if ident == "serde" => match values.first() {
                Some(&MetaItem(NameValue(ref i, Str(_, _)))) if i == "rename" => {
                    true
                }
                _ => false
            },
            _ => false
        });
    serde_struct_rename_attr
}
