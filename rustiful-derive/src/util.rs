use quote::Tokens;
use syn::Body;
use syn::Field;
use syn::Ident;
use syn::Ty;
use syn::VariantData;

/// This is a wrapper for a field, with its ident.
///
/// `Field::ident` returns an `Option<Ident>`, but since we know that there will always be an
/// `Ident` for a given field (since we disallow anything that's not a struct with fields),
/// we return the field and ident in this wrapper. This saves us from having to do any cloning
/// dances when we want to do something with the ident.
pub struct JsonApiField {
    pub field: Field,
    pub ident: Ident
}

pub fn get_attrs_and_id(body: Body) -> (JsonApiField, Vec<JsonApiField>) {
    match body {
        Body::Struct(VariantData::Struct(data)) => {
            let (mut id, attrs): (Vec<JsonApiField>, Vec<JsonApiField>) = data.into_iter()
                .map(|f| {
                    let ident = f.ident
                        .clone()
                        .expect("#[derive(JsonApi)] is not supported for tuple structs");

                    JsonApiField {
                        field: f,
                        ident
                    }
                })
                .partition(|f| {
                    let has_id_ident = f.ident == "id";
                    let has_id_attribute = f.field.attrs.iter().any(|a| a.name() == "JsonApiId");
                    has_id_ident || has_id_attribute
                });

            if id.len() > 1 {
                panic!(
                    "You can only use a JsonApiId attribute or have an id field, not both at \
                     the same time."
                )
            }

            let json_api_id = id.remove(0);

            if is_option_ty(&json_api_id.field.ty) {
                panic!(
                    "Option types are not supported as an id for {}.",
                    &json_api_id.ident
                );
            }

            return (json_api_id, attrs);
        }
        _ => panic!("#[derive(JsonApi)] can only be used with structs")
    }
}

pub fn is_option_ty(ty: &Ty) -> bool {
    let option_ident = Ident::new("Option");
    match *ty {
        Ty::Path(_, ref path) => {
            path.segments
                .first()
                .map(|s| s.ident == option_ident)
                .unwrap_or(false)
        }
        _ => false
    }
}

#[cfg(feature = "uuid")]
/// This method is used to conditionally add the uuid crate to the generated types. If the feature
/// "uuid" is set, then this will add the crate along with a use declaration of `Uuid`.
pub fn get_uuid_tokens() -> Tokens {
    quote! {
        extern crate uuid;
        use self::uuid::Uuid;
    }
}

#[cfg(not(feature = "uuid"))]
/// This method is used to conditionally add the uuid crate to the generated types. If the feature
/// "uuid" is not set, then this does nothing.
pub fn get_uuid_tokens() -> Tokens {
    quote!{}
}
