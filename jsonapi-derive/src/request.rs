extern crate syn;
extern crate quote;
extern crate jsonapi;
extern crate inflector;

use syn::DeriveInput;

use quote::Ident;
use quote::Tokens;

pub fn expand_iron_request_methods(ast: &DeriveInput) -> Tokens {
    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let module_name = Ident::new(format!("__{}", name.to_string().to_lowercase()));

    quote! {
        pub mod #module_name {
            pub mod routes {
                extern crate iron;
                extern crate serde_json;
                extern crate router;

                use self::router::Router;
                use self::iron::prelude::*;
                use self::iron::status;
                use self::iron::headers::ContentType;
                use self::iron::mime::Mime;

                use super::super::#name;
                use std::str::FromStr;

                use jsonapi::queryspec::ToJson;
                use jsonapi::service::ToRequest;
                use jsonapi::service::JsonApiService;
                use jsonapi::queryspec::ToParams;
                use jsonapi::array::JsonApiArray;
                use jsonapi::object::JsonApiObject;
                use jsonapi::data::JsonApiData;

                impl ToRequest<#name> for #name { }

                pub fn get(req: &mut Request) -> IronResult<Response> {
                    let content_type:Mime = "application/vnd.api+json".parse().unwrap();

                    let router = req.extensions
                        .get::<router::Router>()
                        .expect("Expected to get a Router from the request extensions.");

                    let id = router.find("id").unwrap();

                    match <<#name as JsonApiService>::T as ToParams>::Params::from_str(req.url.query().unwrap_or("")) {
                        Ok(params) => {
                            match #name::new().find(&id, &params) {
                                Ok(result) => {
                                    match result {
                                        Some(obj) => {
                                            let data:<<#name as JsonApiService>::T as ToJson>::Resource = (obj, &params).into();
                                            let json = JsonApiObject::<<<#name as JsonApiService>::T as ToJson>::Resource> {
                                                data: data
                                            };

                                            match serde_json::to_string(&json) {
                                                Ok(serialized) => Ok(Response::with((status::Ok, serialized))),
                                                Err(e) => Err(IronError::new(e, status::InternalServerError))
                                            }
                                        },
                                        None => Ok(Response::with((status::NotFound, "")))
                                    }
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

                pub fn index(req: &mut Request) -> IronResult<Response> {
                    let content_type:Mime = "application/vnd.api+json".parse().unwrap();
                    match <<#name as JsonApiService>::T as ToParams>::Params::from_str(req.url.query().unwrap_or("")) {
                        Ok(params) => {
                            match #name::new().find_all(&params) {
                                Ok(result) => {
                                    let data:Vec<_> = result.into_iter().map(|e| {
                                        let json:<<#name as JsonApiService>::T as ToJson>::Resource = (e, &params).into();
                                        json
                                    }).collect();

                                    let json_api_array = JsonApiArray::<<<#name as JsonApiService>::T as ToJson>::Resource> {
                                        data: data
                                    };

                                    match serde_json::to_string(&json_api_array) {
                                        Ok(serialized) => Ok(Response::with((content_type, status::Ok, serialized))),
                                        Err(e) => Err(IronError::new(e, status::InternalServerError))
                                    }
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
