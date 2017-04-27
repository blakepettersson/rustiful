extern crate syn;
extern crate inflector;

use self::inflector::Inflector;
use super::util;
use quote::Ident;
use quote::Tokens;
use syn::DeriveInput;
use syn::Ty;

pub fn expand_json_api_models(ast: &DeriveInput) -> Tokens {
    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let (id, fields) = util::get_attrs_and_id(&ast.body);

    let json_api_id_ty = &id.ty;
    let json_api_id_ident =
        &id.clone().ident.expect("#[derive(JsonApi)] is not supported for tuple structs");
    let generated_jsonapi_attrs = Ident::new(format!("__{}{}", name, "JsonApiAttrs"));

    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let lower_case_name_as_str = lower_case_name.to_string();

    // Used in the quasi-quotation below as `#generated_params_type_name`;
    // append name + `Params` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));

    let attr_fields: Vec<_> = fields.iter()
        .map(|f| {
            let ident =
                f.ident.clone().expect("#[derive(JsonApi)] is not supported for tuple structs");
            (f, ident)
        })
        .collect();

    let jsonapi_attrs: Vec<_> = attr_fields.iter()
        .map(|&(field, ref ident)| {
            let ty = &field.ty;
            if is_option_ty(ty) {
                quote! {
                    #[serde(default, deserialize_with = "rustiful::json_option::some_option")]
                    pub #ident: Option<#ty>
                }
            } else {
                quote!(pub #ident: Option<#ty>)
            }
        })
        .collect();

    let filtered_option_vars: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| quote!(let mut #ident = Some(model.#ident);))
        .collect();

    let filtered_option_fields: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| quote!(#ident: #ident))
        .collect();

    let filtered_option_cases: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| {
            quote! {
                &super::#lower_case_name::field::#ident => #ident = None
            }
        })
        .collect();

    let attr_constructor_fields = filtered_option_fields.clone();

    let jsonapi_builder_id_attr = quote!(pub #json_api_id_ident: #json_api_id_ty);
    let jsonapi_builder_attrs = jsonapi_attrs.clone();
    let jsonapi_builder_id = quote!(#json_api_id_ident: self.#json_api_id_ident);
    let jsonapi_builder_fields: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| {
            let ident_string = &ident.to_string();
            quote! {
                #ident: self.#ident.ok_or(format!("#{} must be initialized", #ident_string))?
            }
        })
        .collect();
    let jsonapi_setter_fields: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| {
            quote! {
                new.#ident = Some(model.#ident);
            }
        })
        .collect();
    let jsonapi_builder_methods: Vec<_> = attr_fields.iter()
        .map(|&(field, ref ident)| {
            let ty = &field.ty;
            quote! {
                pub fn #ident<VALUE: Into<#ty>>(&mut self, value: VALUE) -> &mut Self {
                    let mut new = self;
                    new.#ident = Some(value.into());
                    new
                }
            }
        })
        .collect();

    let jsonapi_builder_setter: Vec<_> = attr_fields.iter()
        .map(|&(_, ref ident)| {
            quote! {
                updated_attrs.attributes.#ident.map(|v| builder.#ident(v));
            }
        })
        .collect();


    let foo: Vec<_> = attr_fields.iter()
        .map(|&(field, ref ident)| {
            let ty = &field.ty;
            quote! {
                #ident: Option<#ty>
            }
        })
        .collect();


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
                pub fn new(#(#foo),*) -> #generated_jsonapi_attrs {
                    #generated_jsonapi_attrs {
                        #(#attr_constructor_fields),*
                    }
                }
            }

            #[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
