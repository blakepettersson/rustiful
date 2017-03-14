use syn::Field;

pub fn get_json_id<'a>(fields: &'a Vec<&'a Field>) -> &'a Field {
    let id = fields.iter().find(|f| f.ident.iter().any(|i| i.to_string() == "id"));
    let json_api_id_attrs: Vec<_> = fields.iter().filter(|f| f.attrs.iter().any(|a| a.name() == "JsonApiId")).collect();

    if json_api_id_attrs.len() > 1 {
        panic!("Invalid: Only one field is allowed to have the JsonApiId attribute!")
    }

    let json_api_attr_id = json_api_id_attrs.first();

    if id != None && json_api_attr_id != None {
        panic!("You can only use a JsonApiId attribute or have an id field, not both at the same \
                time.")
    }

    id.or(json_api_attr_id.cloned())
        .expect("No JsonApiId attribute defined! (or no field named id)")
}
