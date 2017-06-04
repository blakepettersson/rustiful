extern crate syn;
extern crate inflector;

use self::inflector::Inflector;
use quote::Ident;
use quote::Tokens;
use syn::Ty;
use util;
use util::JsonApiField;

pub fn expand_json_api_models(name: &syn::Ident,
                              &(ref id, ref fields): &(JsonApiField, Vec<JsonApiField>))
                              -> Tokens {
    let json_api_id_ty = &id.field.ty;
    let json_api_id_ident = &id.ident;

    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let lower_case_name_as_str = lower_case_name.to_string();

    let mut jsonapi_attrs: Vec<_> = Vec::with_capacity(fields.len());
    let mut filtered_option_vars: Vec<_> = Vec::with_capacity(fields.len());
    let mut filtered_option_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut attr_constructor_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut filtered_option_cases: Vec<_> = Vec::with_capacity(fields.len());
    let mut attr_constructor_args: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_setter: Vec<_> = Vec::with_capacity(fields.len());

    for field in fields {
        let ty = &field.field.ty;
        let ident = &field.ident;

        jsonapi_attrs.push(generate_option_field(ident, ty, true));

        filtered_option_vars.push(quote!(let mut #ident = Some(model.#ident);));

        filtered_option_fields.push(quote!(#ident));
        attr_constructor_fields.push(quote!(#ident: #ident));

        filtered_option_cases.push(quote! {
            &super::#lower_case_name::field::#ident => #ident = None
        });

        jsonapi_builder_setter.push(quote! {
            updated_attrs.attributes.#ident.map(|v| builder.#ident(v));
        });

        attr_constructor_args.push(quote! { #ident: Option<#ty> });
    }

    let mod_name = Ident::new(format!("__json_{}", lower_case_name_as_str));

    let uuid = util::get_uuid_tokens();

    quote! {
        mod #mod_name {
            #uuid

            extern crate rustiful as _rustiful;

            use super::#name;
            use std::str::FromStr;
            use self::_rustiful::ToJson;
            use self::_rustiful::TryFrom;
            use self::_rustiful::TryInto;
            use self::_rustiful::ToBuilder;
            use self::_rustiful::JsonApiData;
            use self::_rustiful::JsonApiBuilder;
            use self::_rustiful::JsonApiResource;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct JsonApiAttributes {
                #(#jsonapi_attrs),*
            }

            impl JsonApiAttributes {
                pub fn new(#(#attr_constructor_args),*) -> JsonApiAttributes {
                    JsonApiAttributes {
                        #(#attr_constructor_fields),*
                    }
                }
            }

            impl TryFrom<JsonApiData<JsonApiAttributes>> for #name {
                type Error = String;

                fn try_from(json: JsonApiData<JsonApiAttributes>) -> Result<Self, Self::Error> {
                    let id = json.id.clone().map(|id| {
                        match #json_api_id_ty::from_str(&id) {
                            Ok(result) => Ok(result),
                            Err(e) => return Err(format!("Failed to parse id value {}: {}", &id, e))
                        }
                    });

                    match id {
                        None => (Default::default(), json).try_into(),
                        Some(Ok(result)) => {
                            (Self {
                                #json_api_id_ident: result,
                                ..Default::default()
                            }, json).try_into()
                        },
                        Some(Err(e)) => Err(e)
                    }
                }
            }

            impl TryFrom<(#name, JsonApiData<JsonApiAttributes>)> for #name {
                type Error = String;

                fn try_from((model, updated_attrs): (#name, JsonApiData<JsonApiAttributes>))
                -> Result<Self, Self::Error> {
                    let mut builder = <#name as ToBuilder>::Builder::new(model);
                    #(#jsonapi_builder_setter)*
                    builder.build()
                }
            }

            impl ToJson for #name {
                type Attrs = JsonApiAttributes;

                fn id(&self) -> String {
                    self.#json_api_id_ident.to_string()
                }

                fn type_name(&self) -> String {
                    <#name as JsonApiResource>::resource_name().to_string()
                }
            }

            /// Converts a `(T, T::Params)` to a `JsonApiAttributes`.
            ///
            /// If `params.filter.fields` is empty, this will set each field of the
            /// `JsonApiAttributes` to whatever value that the `model` field has.
            ///
            /// However, `params.filter.fields` is not empty, all fields that are not present
            /// in `params.filter.fields` will be set to `None.` With this, we only serialize the
            /// fields that we want to display when fetching an object.
            impl <'a> From<(#name, &'a <#name as JsonApiResource>::Params)> for JsonApiAttributes {
                fn from((model, params): (#name, &'a <#name as JsonApiResource>::Params)) -> Self {
                    #(#filtered_option_vars)*

                    let fields = &params.fieldset.fields;
                    if !fields.is_empty() {
                        for field in super::#lower_case_name::field::iter() {
                            if !fields.contains(field) {
                                match field {
                                    #(#filtered_option_cases),*
                                }
                            }
                        }
                    }

                    JsonApiAttributes::new(#(#filtered_option_fields),*)
                }
            }
        }
    }
}

pub fn generate_option_field(ident: &syn::Ident, ty: &Ty, generate_serde_attribute: bool) -> Tokens {
    if util::is_option_ty(ty) && generate_serde_attribute {
        quote! {
                #[serde(default, deserialize_with = "self::_rustiful::json_option::some_option")]
                pub #ident: Option<#ty>
        }
    } else {
        quote!(pub #ident: Option<#ty>)
    }
}
