#![crate_type = "proc-macro"]
#![recursion_limit = "512"]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![warn(missing_debug_implementations, missing_copy_implementations, trivial_casts,
trivial_numeric_casts, unused_import_braces, unused_qualifications)]

#[macro_use]
extern crate quote;

extern crate syn;
extern crate proc_macro;

mod util;
mod json;
mod params;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(JsonApi, attributes(JsonApiId))]
pub fn generate_json_api(input: TokenStream) -> TokenStream {

    let source = parse_derive_input(&input);

    // Build the output
    let mut expanded = params::expand_json_api_fields(&source);
    expanded.append(json::expand_json_api_models(&source).as_str());

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}

#[proc_macro_derive(JsonApiResource, attributes(JsonApiId))]
pub fn generate_json_api_models(input: TokenStream) -> TokenStream {
    json::expand_json_api_models(&parse_derive_input(&input)).parse().unwrap()
}

#[proc_macro_derive(JsonApiParams)]
pub fn generate_json_api_request_parameters(input: TokenStream) -> TokenStream {
    params::expand_json_api_fields(&parse_derive_input(&input)).parse().unwrap()
}

fn parse_derive_input(input: &TokenStream) -> DeriveInput {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    syn::parse_derive_input(&source).unwrap()
}
