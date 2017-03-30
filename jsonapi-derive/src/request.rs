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

                use super::super::#name;
                use self::iron::prelude::*;
                use jsonapi::request::FromRequest;
                use jsonapi::service::JsonApiService;
                use jsonapi::iron::GetHandler;
                use jsonapi::iron::IndexHandler;
                use jsonapi::iron::IronHandlers;
                use jsonapi::params::JsonApiResource;

                pub struct _GetHandler {}
                pub struct _IndexHandler {}

                impl <'a> FromRequest<'a, <#name as JsonApiService>::T, <<#name as JsonApiService>::T as JsonApiResource>::Params, <<#name as JsonApiService>::T as JsonApiResource>::SortField, <<#name as JsonApiService>::T as JsonApiResource>::FilterField> for #name {}
                impl <'a> GetHandler<'a, #name, <#name as JsonApiService>::T, <<#name as JsonApiService>::T as JsonApiResource>::Params, <<#name as JsonApiService>::T as JsonApiResource>::SortField, <<#name as JsonApiService>::T as JsonApiResource>::FilterField> for _GetHandler {}
                impl <'a> IndexHandler<'a, #name, <#name as JsonApiService>::T, <<#name as JsonApiService>::T as JsonApiResource>::Params, <<#name as JsonApiService>::T as JsonApiResource>::SortField, <<#name as JsonApiService>::T as JsonApiResource>::FilterField> for _IndexHandler {}

                impl iron::Handler for _GetHandler {
                    fn handle(&self, req: &mut Request) -> IronResult<Response> {
                        <_GetHandler as GetHandler<#name, <#name as JsonApiService>::T, <<#name as JsonApiService>::T as JsonApiResource>::Params, <<#name as JsonApiService>::T as JsonApiResource>::SortField, <<#name as JsonApiService>::T as JsonApiResource>::FilterField>>::get(req)
                    }
                }

                impl iron::Handler for _IndexHandler {
                    fn handle(&self, req: &mut Request) -> IronResult<Response> {
                        <_IndexHandler as IndexHandler<#name, <#name as JsonApiService>::T, <<#name as JsonApiService>::T as JsonApiResource>::Params, <<#name as JsonApiService>::T as JsonApiResource>::SortField, <<#name as JsonApiService>::T as JsonApiResource>::FilterField>>::index(req)
                    }
                }

                impl <'a> IronHandlers<'a, #name> for #name {
                    type Params = <<#name as JsonApiService>::T as JsonApiResource>::Params;
                    type Resource = <#name as JsonApiService>::T;
                    type SortField = <<#name as JsonApiService>::T as JsonApiResource>::SortField;
                    type FilterField = <<#name as JsonApiService>::T as JsonApiResource>::FilterField;
                    type GetHandler = _GetHandler;
                    type IndexHandler = _IndexHandler;
                }
            }
        }
    }
}
