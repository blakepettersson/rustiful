use syn::Body;
use syn::Field;
use syn::Ident;
use syn::VariantData;
use quote::Tokens;

pub struct JsonApiField {
    pub field: Field,
    pub ident: Ident,
}

pub fn get_attrs_and_id(body: Body) -> (JsonApiField, Vec<JsonApiField>) {
    match body {
        Body::Struct(VariantData::Struct(data)) => {
            let (id, attrs): (Vec<JsonApiField>, Vec<JsonApiField>) = data.into_iter()
                .map(|f| {
                    let ident = f.ident
                        .clone()
                        .expect("#[derive(JsonApi)] is not supported for tuple structs");

                    JsonApiField {
                        field: f,
                        ident: ident,
                    }
                })
                .partition(|f| {
                    let has_id_ident = f.ident == "id";
                    let has_id_attribute = f.field.attrs.iter().any(|a| a.name() == "JsonApiId");
                    has_id_ident || has_id_attribute
                });

            if id.len() > 1 {
                panic!("You can only use a JsonApiId attribute or have an id field, not both at \
                the same time.")
            }

            // This seems to be the only way to get the first element by value in stable Rust.
            for json_api_id in id {
                return (json_api_id, attrs);
            }

            panic!("No JsonApiId attribute defined (or no field named id)!")
        }
        _ => panic!("#[derive(JsonApi)] can only be used with structs"),
    }
}

#[cfg(feature = "uuid")]
/// This method is used to conditionally add the uuid crate to the generated types. . If the feature
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
    quote! {}
}
