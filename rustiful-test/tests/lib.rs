#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rustiful_derive;

extern crate rustiful;

use rustiful::SortOrder::*;
use rustiful::JsonApiResource;
use rustiful::QueryStringParseError;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
struct Foo {
    #[JsonApiId]
    bar: i32,
    foo: i32,
    abc: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
#[serde(rename = "renamed")]
struct Bar {
    id: String,
    bar: i32,
}

#[test]
fn parse_renamed_json_struct() {
    use self::bar::field::*;
    match <Bar as JsonApiResource>::from_str("fields[renamed]=bar") {
        Ok(result) => assert_eq!(Some(&bar), result.filter.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_renamed_json_struct_fails_on_original_name() {
    match <Bar as JsonApiResource>::from_str("fields[bar]=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => assert_eq!(QueryStringParseError::UnImplementedError, e),
    }
}

#[test]
fn parse_params_fails_on_id_param() {
    match <Bar as JsonApiResource>::from_str("fields[renamed]=id") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidValue("Invalid field: id".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_present_field() {
    use self::foo::field::*;
    match <Foo as JsonApiResource>::from_str("fields[foo]=foo") {
        Ok(result) => assert_eq!(Some(&foo), result.filter.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_field_that_is_not_present() {
    match <Foo as JsonApiResource>::from_str("") {
        Ok(result) => assert_eq!(true, result.filter.fields.is_empty()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_fields_fails_if_query_param_is_not_valid() {
    match <Foo as JsonApiResource>::from_str("fields=body=foo") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidParam("fields=body=foo".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_fields_fails_if_fields_value_is_empty() {
    match <Foo as JsonApiResource>::from_str("fields[foo]=") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidValue("Fields for foo are empty".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_fields_fails_if_field_value_contains_field_that_does_not_exist() {
    match <Foo as JsonApiResource>::from_str("fields[foo]=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidValue("Invalid field: non_existent"
                           .to_string()),
                       e)
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_doesnt_contain_left_bracket() {
    match <Foo as JsonApiResource>::from_str("fieldsarticles]=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidKeyParam("articles]".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_does_not_contain_right_bracket() {
    match <Foo as JsonApiResource>::from_str("fields[articles=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidKeyParam("[articles".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::from_str("sort=foo") {
        Ok(result) => assert_eq!(Some(&foo(Asc)), result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_descending_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::from_str("sort=-foo") {
        Ok(result) => assert_eq!(Some(&foo(Desc)), result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_multiple_sort_fields() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::from_str("sort=-foo,abc") {
        Ok(result) => {
            let expected = vec![foo(Desc), abc(Asc)];
            assert_eq!(expected, result.sort.fields)
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_null_sort_field() {
    match <Foo as JsonApiResource>::from_str("") {
        Ok(result) => assert_eq!(None, result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}


#[test]
fn parse_query_param() {
    match <Foo as JsonApiResource>::from_str("foo=bar") {
        Ok(result) => {
            let expected = &"bar".to_string();
            assert_eq!(Some(expected), result.query_params.get("foo"))
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_sort_field_fails_on_id_param() {
    match <Foo as JsonApiResource>::from_str("sort=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidValue("Invalid field: bar".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_sort_field_fails_on_non_existent_param() {
    match <Foo as JsonApiResource>::from_str("sort=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidValue("Invalid field: non_existent"
                           .to_string()),
                       e)
        }
    }
}