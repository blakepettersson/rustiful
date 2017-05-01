extern crate syn;
extern crate inflector;

use self::inflector::Inflector;
use quote::Ident;
use quote::Tokens;
use syn::Ty;
use util::JsonApiField;

pub fn expand_json_api_models(name: &syn::Ident,
                              &(ref id, ref fields): &(JsonApiField, Vec<JsonApiField>))
                              -> Tokens {
    let json_api_id_ty = &id.field.ty;
    let json_api_id_ident = &id.ident;
    let generated_jsonapi_attrs = Ident::new(format!("__{}{}", name, "JsonApiAttrs"));

    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let lower_case_name_as_str = lower_case_name.to_string();

    // Used in the quasi-quotation below as `#generated_params_type_name`;
    // append name + `Params` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));

    let mut jsonapi_attrs: Vec<_> = Vec::with_capacity(fields.len());
    let mut filtered_option_vars: Vec<_> = Vec::with_capacity(fields.len());
    let mut filtered_option_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut attr_constructor_fields: Vec<_> = Vec::with_capacity(fields.len());


    let mut filtered_option_cases: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_attrs: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_setter_fields: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_methods: Vec<_> = Vec::with_capacity(fields.len());
    let mut jsonapi_builder_setter: Vec<_> = Vec::with_capacity(fields.len());
    let mut attr_constructor_args: Vec<_> = Vec::with_capacity(fields.len());

    for field in fields {
        let ty = &field.field.ty;
        let ident = &field.ident;
        let ident_string = &ident.to_string();

        jsonapi_attrs.push(generate_option_method(ident, ty, true));
        jsonapi_builder_attrs.push(generate_option_method(ident, ty, false));

        filtered_option_vars.push(quote!(let mut #ident = Some(model.#ident);));

        let fields = quote!(#ident: #ident);
        filtered_option_fields.push(fields.clone());
        attr_constructor_fields.push(fields);

        filtered_option_cases.push(quote! {
            &super::#lower_case_name::field::#ident => #ident = None
        });
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

        attr_constructor_args.push(quote! { #ident: Option<#ty> });
    }

    let jsonapi_builder_id_attr = quote!(pub #json_api_id_ident: #json_api_id_ty);
    let jsonapi_builder_id = quote!(#json_api_id_ident: self.#json_api_id_ident);

    let mod_name = Ident::new(format!("__json_{}", lower_case_name_as_str));

    quote! {
        mod #mod_name {
            extern crate rustiful;

            use super::#name;
            use super::#lower_case_name::#generated_params_type_name;

            use rustiful::ToJson;
            use rustiful::TryFrom;
            use rustiful::JsonApiId;
            use rustiful::JsonApiData;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct #generated_jsonapi_attrs {
                #(#jsonapi_attrs),*
            }

            impl #generated_jsonapi_attrs {
                pub fn new(#(#attr_constructor_args),*) -> #generated_jsonapi_attrs {
                    #generated_jsonapi_attrs {
                        #(#attr_constructor_fields),*
                    }
                }
            }

            #[derive(Debug, Default, Clone, PartialEq, Eq)]
            struct Builder {
                #jsonapi_builder_id_attr,
                #(#jsonapi_builder_attrs),*
            }

            impl Builder {
                #(#jsonapi_builder_methods)*

                pub fn new(model: #name) -> Self {
                    let mut new:Self = Default::default();
                    #(#jsonapi_setter_fields)*
                    new
                }

                pub fn build(self) -> Result<#name, String> {
                    Ok(#name {
                        #jsonapi_builder_id,
                        #(#jsonapi_builder_fields),*
                    })
                }
            }

            impl TryFrom<(#name, JsonApiData<#generated_jsonapi_attrs>)> for #name {
                type Error = String;

                fn try_from(pair: (#name, JsonApiData<#generated_jsonapi_attrs>)) ->
                Result<Self, Self::Error> {
                    let (model, updated_attrs) = pair;
                    let mut builder = Builder::new(model);
                    #(#jsonapi_builder_setter)*
                    builder.build()
                }
            }

            impl ToJson for #name {
                type Attrs = #generated_jsonapi_attrs;
                type Resource = JsonApiData<#generated_jsonapi_attrs>;

                fn id(&self) -> JsonApiId {
                    self.#json_api_id_ident.clone().into()
                }

                fn type_name(&self) -> String {
                    #lower_case_name_as_str.to_string()
                }
            }

            impl <'a> From<(#name, &'a #generated_params_type_name)> for #generated_jsonapi_attrs {
                fn from(pair: (#name, &'a #generated_params_type_name)) -> Self {
                    let (model, params) = pair;

                    #(#filtered_option_vars)*

                    let fields = &params.filter.fields;
                    if !fields.is_empty() {
                        for field in super::#lower_case_name::field::iter() {
                            if !fields.contains(field) {
                                match field {
                                    #(#filtered_option_cases),*
                                }
                            }
                        }
                    }

                    #generated_jsonapi_attrs {
                        #(#filtered_option_fields),*
                    }
                }
            }
        }
    }
}

fn generate_option_method(ident: &syn::Ident, ty: &Ty, generate_serde_attribute: bool) -> Tokens {
    if is_option_ty(ty) && generate_serde_attribute {
        quote! {
                #[serde(default, deserialize_with = "rustiful::json_option::some_option")]
                pub #ident: Option<#ty>
        }
    } else {
        quote!(pub #ident: Option<#ty>)
    }
}

fn is_option_ty(ty: &Ty) -> bool {
    let option_ident = Ident::new("Option");
    match *ty {
        Ty::Path(_, ref path) => {
            path.segments
                .first()
                .map(|s| s.ident == option_ident)
                .unwrap_or(false)
        }
        _ => false,
    }
}
