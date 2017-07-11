
use json::generate_option_field;
use quote::Tokens;
use syn::Ident;
use util;
use util::JsonApiField;
extern crate inflector;

use self::inflector::Inflector;

pub fn expand_json_api_builders(
    name: &Ident,
    &(ref id, ref fields): &(JsonApiField, Vec<JsonApiField>)
) -> Tokens {
    let json_api_id_ty = &id.field.ty;
    let json_api_id_ident = &id.ident;
    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let lower_case_name_as_str = lower_case_name.to_string();

    let mut jsonapi_attrs: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_attrs: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_setter_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_methods: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_setter: Vec<_> = Vec::with_capacity(fields.len());

    for field in fields {
        let ty = &field.field.ty;
        let ident = &field.ident;
        let ident_string = &ident.to_string();

        jsonapi_attrs.push(generate_option_field(ident, ty, true));
        jsonapi_builder_attrs.push(generate_option_field(ident, ty, false));

        jsonapi_builder_fields.push(quote! {
            #ident: self.#ident.ok_or(format!("#{} must be initialized", #ident_string))?
        });
        jsonapi_setter_fields.push(quote! { new.#ident = Some(model.#ident); });
        jsonapi_builder_methods.push(quote! {
            pub fn #ident<VALUE: Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
                let mut new = self;
                new.#ident = Some(value.into());
                new
            }
        });

        jsonapi_builder_setter.push(quote! {
            updated_attrs.attributes.#ident.map(|v| builder.#ident(v));
        });
    }

    let jsonapi_builder_id_attr = quote!(pub #json_api_id_ident: #json_api_id_ty);
    let jsonapi_builder_id = quote!(#json_api_id_ident: self.#json_api_id_ident);
    let jsonapi_builder_id_setter = quote!(new.#json_api_id_ident = model.#json_api_id_ident;);

    let mod_name = Ident::new(format!("__builder_{}", lower_case_name_as_str));

    let uuid = util::get_uuid_tokens();

    quote! {
        mod #mod_name {
            #uuid

            extern crate rustiful as _rustiful;

            use super::#name;
            use self::_rustiful::ToBuilder;
            use self::_rustiful::JsonApiBuilder;

            #[derive(Debug, Default)]
            pub struct Builder {
                #jsonapi_builder_id_attr,
                #(#jsonapi_builder_attrs),*
            }

            impl Builder {
                #(#jsonapi_builder_methods)*
            }

            impl JsonApiBuilder<#name> for Builder {
                fn new(model: #name) -> Self {
                    let mut new:Self = Default::default();
                    #jsonapi_builder_id_setter
                    #(#jsonapi_setter_fields)*
                    new
                }

                fn build(self) -> Result<#name, String> {
                    Ok(#name {
                        #jsonapi_builder_id,
                        #(#jsonapi_builder_fields),*
                    })
                }
            }

            impl ToBuilder for #name {
                type Builder = Builder;
            }
        }
    }
}
