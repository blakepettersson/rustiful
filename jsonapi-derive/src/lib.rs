#![crate_type = "proc-macro"]
#![recursion_limit = "512"]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate quote;

extern crate syn;
extern crate proc_macro;

mod params;

use proc_macro::TokenStream;

#[proc_macro_derive(JsonApi, attributes(JsonApiId))]
pub fn json_api(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = params::expand_json_api_fields(&ast);

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}