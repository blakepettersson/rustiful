extern crate syn;
extern crate quote;
extern crate jsonapi;
extern crate inflector;

use syn::DeriveInput;

use quote::Ident;
use quote::Tokens;
use self::inflector::Inflector;

pub fn expand_iron_request_methods(ast: &DeriveInput) -> Tokens {
    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let module_name = Ident::new(format!("__{}", name.to_string().to_lowercase()));

    quote! {
        pub mod #module_name {
            pub mod routes {
                extern crate iron;
                extern crate serde_json;

                use self::iron::prelude::*;
                use self::iron::status;

                use super::super::#name;
                use std::str::FromStr;
                use jsonapi::queryspec::ToJson;
                use jsonapi::service::ToRequest;
                use jsonapi::service::JsonApiService;
                use jsonapi::queryspec::ToParams;
                use jsonapi::jsonapi_array::JsonApiArray;

                impl ToRequest<#name> for #name { }

                pub fn index(req: &mut Request) -> IronResult<Response> {
                    match <<#name as JsonApiService>::T as ToParams>::Params::from_str(req.url.query().unwrap_or("")) {
                        Ok(params) => {
                            match #name::new().find_all(&params) {
                                Ok(result) => {
                                    let mapped:Vec<_> = result.into_iter().map(|e| {
                                        let json:<<#name as JsonApiService>::T as ToJson>::Json = (e, &params).into();
                                        json
                                    }).collect();

                                    let json_api_array = JsonApiArray::<<<#name as JsonApiService>::T as ToJson>::Json> {
                                        data: mapped
                                    };
                                    let serialized = serde_json::to_string(&json_api_array).unwrap();
                                    Ok(Response::with((status::Ok, serialized)))
                                },
                                Err(e) => {
                                    Err(IronError::new(e, status::InternalServerError))
                                }
                            }
                        },
                        Err(e) => {
                            Err(IronError::new(e, status::InternalServerError))
                        },
                    }
                }
            }
        }
    }
}
