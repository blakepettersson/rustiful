#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate rustiful_derive;

extern crate rustiful;

use rustiful::IntoJson;
use rustiful::JsonApiData;
use rustiful::JsonApiResource;
use rustiful::QueryStringParseError;
use rustiful::SortOrder::*;
use std::str::FromStr;

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
struct Foo {
    #[JsonApiId]
    bar: i32,
    foo: i32,
    abc: String,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
#[serde(rename = "renamed")]
struct Bar {
    id: String,
    bar: i32,
}

#[test]
fn parse_renamed_json_struct() {
    use self::bar::field::*;
    match <Bar as JsonApiResource>::Params::from_str("fields[renamed]=bar") {
        Ok(result) => assert_eq!(Some(&bar), result.fieldset.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_renamed_json_struct_fails_on_original_name() {
    match <Bar as JsonApiResource>::Params::from_str("fields[bar]=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => assert_eq!(QueryStringParseError::UnImplementedError, e),
    }
}

#[test]
fn parse_params_fails_on_id_param() {
    match <Bar as JsonApiResource>::Params::from_str("fields[renamed]=id") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => assert_eq!(QueryStringParseError::InvalidFieldValue("id".to_string()), e),
    }
}

#[test]
fn parse_present_field() {
    use self::foo::field::*;
    match <Foo as JsonApiResource>::Params::from_str("fields[foos]=foo") {
        Ok(result) => assert_eq!(Some(&foo), result.fieldset.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_present_url_encoded_field() {
    use self::foo::field::*;
    match <Foo as JsonApiResource>::Params::from_str("fields%5Bfoos%5D=foo") {
        Ok(result) => assert_eq!(Some(&foo), result.fieldset.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_field_that_is_not_present() {
    match <Foo as JsonApiResource>::Params::from_str("") {
        Ok(result) => assert_eq!(true, result.fieldset.fields.is_empty()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_fields_fails_if_query_param_is_not_valid() {
    match <Foo as JsonApiResource>::Params::from_str("fields=body=foo") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => assert_eq!(QueryStringParseError::InvalidFieldsetKey("".to_string()), e),
    }
}

#[test]
fn parse_fields_fails_if_fields_value_is_empty() {
    match <Foo as JsonApiResource>::Params::from_str("fields[foo]=") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::EmptyFieldsetValue("foo".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_fields_fails_if_field_value_contains_field_that_does_not_exist() {
    match <Foo as JsonApiResource>::Params::from_str("fields[foos]=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidFieldValue("non_existent".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_doesnt_contain_left_bracket() {
    match <Foo as JsonApiResource>::Params::from_str("fieldsarticles]=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidFieldsetKey("articles]".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_does_not_contain_right_bracket() {
    match <Foo as JsonApiResource>::Params::from_str("fields[articles=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidFieldsetKey("[articles".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=foo") {
        Ok(result) => assert_eq!(Some(&foo(Asc)), result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_descending_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=-foo") {
        Ok(result) => assert_eq!(Some(&foo(Desc)), result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_multiple_sort_fields() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=-foo,abc") {
        Ok(result) => {
            let expected = vec![foo(Desc), abc(Asc)];
            assert_eq!(expected, result.sort.fields)
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_null_sort_field() {
    match <Foo as JsonApiResource>::Params::from_str("") {
        Ok(result) => assert_eq!(None, result.sort.fields.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_query_param() {
    match <Foo as JsonApiResource>::Params::from_str("foo=bar") {
        Ok(result) => {
            let expected = vec!["bar".to_string()];
            assert_eq!(&expected, result.query_params.get("foo").unwrap())
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_multiple_query_params() {
    match <Foo as JsonApiResource>::Params::from_str("foo=bar&foo=test") {
        Ok(result) => {
            let expected = vec!["bar".to_string(), "test".to_string()];
            assert_eq!(&expected, result.query_params.get("foo").unwrap())
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e)),
    }
}

#[test]
fn parse_sort_field_fails_on_id_param() {
    match <Foo as JsonApiResource>::Params::from_str("sort=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => assert_eq!(QueryStringParseError::InvalidSortValue("bar".to_string()), e),
    }
}

#[test]
fn parse_sort_field_fails_on_multiple_sort_params() {
    match <Foo as JsonApiResource>::Params::from_str("sort=foo&sort=foo") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::DuplicateSortKey("foo".to_string()),
                       e)
        }
    }
}

#[test]
fn parse_sort_field_fails_on_non_existent_param() {
    match <Foo as JsonApiResource>::Params::from_str("sort=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            assert_eq!(QueryStringParseError::InvalidSortValue("non_existent".to_string()),
                       e)
        }
    }
}

#[test]
fn test_into_conversions_with_int_id() {
    let test = Foo {
        bar: 1,
        foo: 2,
        abc: "abc".to_string(),
    };

    let expected_id = test.bar.to_string();
    let result: JsonApiData<Foo> = test.into_json(&Default::default());
    assert_eq!(expected_id, result.id.expect("unexpected None on id!"));
    assert_eq!(2, result.attributes.foo.expect("unexpected None on foo!"));
    assert_eq!("abc".to_string(),
               result.attributes.abc.expect("unexpected None on abc!"));
}

#[test]
fn test_into_conversions_with_string_id() {
    let test = Bar {
        id: "test".to_string(),
        bar: 1,
    };

    let expected_id = test.id.clone();
    let result: JsonApiData<Bar> = test.into_json(&Default::default());
    assert_eq!(expected_id, result.id.expect("unexpected None on id!"));
    assert_eq!(1, result.attributes.bar.expect("unexpected None on bar!"));
}
