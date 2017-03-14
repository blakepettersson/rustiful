extern crate syn;

use super::util;
use syn::Ty;
use syn::DeriveInput;
use quote::Ident;
use quote::Tokens;

pub fn expand_json_api_models(ast: &DeriveInput) -> Tokens {
    // Used in the quasi-quotation below as `#name`
    let name = &ast.ident;

    let fields: Vec<_> = match ast.body {
        syn::Body::Struct(ref data) => data.fields().iter().collect(),
        syn::Body::Enum(_) => panic!("#[derive(JsonApi)] can only be used with structs"),
    };

    let json_api_id = util::get_json_id(&fields);
    let json_api_id_ty = &json_api_id.ty;
    let generated_jsonapi_attrs = Ident::new(format!("__{}{}", name, "JsonApiAttrs"));
    let generated_jsonapi_resource = Ident::new(format!("__{}{}", name, "JsonApiResource"));

    let attr_fields: Vec<_> = fields.iter().filter(|f| **f != json_api_id).collect();

    let jsonapi_attrs: Vec<_> = attr_fields.iter()
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;
            let option_ty = inner_of_option_ty(ty).unwrap_or(ty);

            quote!(#ident: #option_ty)
        })
        .collect();

    quote! {
        pub struct #generated_jsonapi_attrs {
            #(#jsonapi_attrs),*
        }

        pub struct #generated_jsonapi_resource {
            id: #json_api_id_ty,
            lower_case_type: String,
            attributes: #generated_jsonapi_attrs
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
