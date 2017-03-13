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

    let prefixed_index = Ident::new(format!("__{}_{}", name.to_string().to_snake_case(), "index"));

    quote! {
        pub mod routes {
            extern crate iron;
            extern crate serde_json;

            use self::iron::prelude::*;
            use self::iron::status;

            use super::#name;
            use std::str::FromStr;
            use jsonapi::service::ToRequest;
            use jsonapi::service::JsonApiService;
            use jsonapi::queryspec::ToParams;

            impl ToRequest<#name> for #name { }

            fn #prefixed_index(req: &mut Request) -> IronResult<Response> {
                match <<#name as JsonApiService>::T as ToParams>::Params::from_str(req.url.query().unwrap_or("")) {
                    Ok(params) => {
                        match #name::new().find_all(params) {
                            Ok(params) => {
                                let serialized = serde_json::to_string(&params).unwrap();
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