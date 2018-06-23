use resources::simple_resources::*;
use rustiful::JsonApiResource;
use rustiful::QueryStringParseError;
use rustiful::SortOrder::*;
use std::str::FromStr;

#[test]
fn parse_renamed_json_struct() {
    use self::bar::field::*;
    match <Bar as JsonApiResource>::Params::from_str("fields[renamed]=bar") {
        Ok(result) => assert_eq!(Some(&bar), result.fields.bar.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_renamed_json_struct_fails_on_original_name() {
    match <Bar as JsonApiResource>::Params::from_str("fields[bar]=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {} //assert_eq!(QueryStringParseError::UnImplementedError, e)
    }
}

#[test]
fn parse_params_fails_on_id_param() {
    match <Bar as JsonApiResource>::Params::from_str("fields[renamed]=id") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::InvalidFieldValue("id".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_present_field() {
    use self::foo::field::*;
    match <Foo as JsonApiResource>::Params::from_str("fields[foos]=foo") {
        Ok(result) => assert_eq!(Some(&foo), result.fields.foo.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_present_url_encoded_field() {
    use self::foo::field::*;
    match <Foo as JsonApiResource>::Params::from_str("fields%5Bfoos%5D=foo") {
        Ok(result) => assert_eq!(Some(&foo), result.fields.foo.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_field_that_is_not_present() {
    match <Foo as JsonApiResource>::Params::from_str("") {
        Ok(result) => assert_eq!(true, result.fields.foo.is_empty()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_fields_fails_if_query_param_is_not_valid() {
    match <Foo as JsonApiResource>::Params::from_str("fields=body=foo") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {}//assert_eq!(QueryStringParseError::InvalidfieldsKey("".to_string()), e)
    }
}

#[test]
fn parse_fields_fails_if_fields_value_is_empty() {
    match <Foo as JsonApiResource>::Params::from_str("fields[foo]=") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::EmptyfieldsValue("foo".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_fields_fails_if_field_value_contains_field_that_does_not_exist() {
    match <Foo as JsonApiResource>::Params::from_str("fields[foos]=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
           /* assert_eq!(
                QueryStringParseError::InvalidFieldValue("non_existent".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_doesnt_contain_left_bracket() {
    match <Foo as JsonApiResource>::Params::from_str("fieldsarticles]=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::InvalidfieldsKey("articles]".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_single_field_fails_if_field_does_not_contain_right_bracket() {
    match <Foo as JsonApiResource>::Params::from_str("fields[articles=title") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::InvalidfieldsKey("[articles".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=foo") {
        Ok(result) => assert_eq!(Some(&foo(Asc)), result.sort.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_descending_sort_field() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=-foo") {
        Ok(result) => assert_eq!(Some(&foo(Desc)), result.sort.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_multiple_sort_fields() {
    use self::foo::sort::*;
    match <Foo as JsonApiResource>::Params::from_str("sort=-foo,abc") {
        Ok(result) => {
            let expected = vec![foo(Desc), abc(Asc)];
            assert_eq!(expected, result.sort)
        }
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_null_sort_field() {
    match <Foo as JsonApiResource>::Params::from_str("") {
        Ok(result) => assert_eq!(None, result.sort.first()),
        Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
    }
}

#[test]
fn parse_sort_field_fails_on_id_param() {
    match <Foo as JsonApiResource>::Params::from_str("sort=bar") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
           /* assert_eq!(
                QueryStringParseError::InvalidSortValue("bar".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_sort_field_fails_on_multiple_sort_params() {
    match <Foo as JsonApiResource>::Params::from_str("sort=foo&sort=foo") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::DuplicateSortKey("foo".to_string()),
                e
            )*/
        }
    }
}

#[test]
fn parse_sort_field_fails_on_non_existent_param() {
    match <Foo as JsonApiResource>::Params::from_str("sort=non_existent") {
        Ok(_) => assert!(false, "expected error but no error happened!"),
        Err(e) => {
            /*assert_eq!(
                QueryStringParseError::InvalidSortValue("non_existent".to_string()),
                e
            )*/
        }
    }
}

