extern crate syn;
extern crate inflector;

use super::util;
use syn::Ty;
use syn::DeriveInput;
use quote::Ident;
use quote::Tokens;
use self::inflector::Inflector;

pub fn expand_json_api_models(ast: &DeriveInput) -> Tokens {
    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().collect(),
        syn::Body::Enum(_) => panic!("#[derive(JsonApi)] can only be used with structs"),
    };

    let json_api_id = util::get_json_id(&fields);
    let json_api_id_ident = &json_api_id.ident;
    let generated_jsonapi_attrs = Ident::new(format!("__{}{}", name, "JsonApiAttrs"));

    let lower_case_name = Ident::new(name.to_string().to_snake_case());
    let lower_case_name_as_str = lower_case_name.to_string();
    // Used in the quasi-quotation below as `#generated_field_type_name`;
    // append name + `Fields` to the new struct name
    let generated_params_type_name = Ident::new(format!("__{}{}", name, "Params"));
    let attr_fields: Vec<_> = fields.iter().filter(|f| **f != json_api_id).collect();

    let jsonapi_attrs: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            let option_ty = inner_of_option_ty(ty).unwrap_or(ty);
            quote!(#ident: Option<#option_ty>)
        })
        .collect();

    let filtered_option_vars: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            let model_value = Ident::new(format!("model.{}", ident.clone().expect("fail").to_string()));
            quote!(let mut #ident = Some(#model_value);)
        })
        .collect();

    let filtered_option_fields: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            quote!(#ident: #ident)
        })
        .collect();

    let filtered_option_cases: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident.clone().expect("fail");
            let enum_variant = Ident::new(format!("&super::{}::field::{}", lower_case_name.to_string(), ident));
            quote! {
                #enum_variant => #ident = None
            }
        })
        .collect();

    let model_id_field = Ident::new(format!("self.{}", json_api_id_ident.clone().expect("fail").to_string()));

    let mod_name = Ident::new(format!("__json_{}", lower_case_name_as_str));

    quote! {
        mod #mod_name {
            use super::#name;
            use super::#lower_case_name::#generated_params_type_name;

            use jsonapi::ToJson;
            use jsonapi::JsonApiId;
            use jsonapi::JsonApiData;

            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct #generated_jsonapi_attrs {
                #(#jsonapi_attrs),*
            }

            impl ToJson for #name {
                type Attrs = #generated_jsonapi_attrs;
                type Resource = JsonApiData<#generated_jsonapi_attrs>;

                fn id(&self) -> JsonApiId {
                    #model_id_field.clone().into()
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

fn inner_of_option_ty(ty: &Ty) -> Option<&Ty> {
    use self::syn::PathParameters::AngleBracketed;

    if !is_option_ty(ty) {
        return None;
    }

    match *ty {
        Ty::Path(_, syn::Path { ref segments, .. }) => {
            match segments[0].parameters {
                AngleBracketed(ref data) => data.types.first(),
                _ => None,
            }
        }
        _ => None,
    }
}
