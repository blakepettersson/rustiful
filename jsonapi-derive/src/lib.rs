#![crate_type = "proc-macro"]
#![recursion_limit = "512"]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate quote;

extern crate syn;
extern crate proc_macro;

mod params;
mod request;

use proc_macro::TokenStream;

#[proc_macro_derive(JsonApi, attributes(JsonApiId))]
pub fn generate_json_api_models(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = params::expand_json_api_fields(&ast);

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}

#[proc_macro_derive(JsonApiRepository, attributes(resource))]
pub fn generate_jsonapi_req_handlers(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = request::expand_iron_request_methods(&ast);

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}