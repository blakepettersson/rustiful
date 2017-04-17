use syn::Body;
use syn::Field;

pub fn get_attrs_and_id(body: &Body) -> (&Field, Vec<&Field>) {
    match *body {
        Body::Struct(ref data) => {
            let (id, attrs): (Vec<&Field>, Vec<&Field>) =
                data.fields().into_iter().partition(|f| {
                    let has_id_ident = f.ident.iter().any(|i| i == "id");
                    let has_id_attribute = f.attrs.iter().any(|a| a.name() == "JsonApiId");
                    has_id_ident || has_id_attribute
                });

            if id.len() > 1 {
                panic!("You can only use a JsonApiId attribute or have an id field, not both at \
                the same time.")
            }

            let json_api_id = id.first().expect("No JsonApiId attribute defined! \
            (or no field named id)");

            (json_api_id, attrs)
        }
        _ => panic!("#[derive(JsonApi)] can only be used with structs"),
    }
}
