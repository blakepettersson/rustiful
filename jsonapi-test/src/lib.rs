#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate jsonapi_derive;

extern crate jsonapi;

#[cfg(test)]
mod tests {
    extern crate jsonapi;

    use self::foo::sort::*;

    use std::str::FromStr;
    use jsonapi::sort_order::SortOrder::*;
    use jsonapi::queryspec::ToParams;
    use jsonapi::queryspec::QueryStringParseError;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
    struct Foo {
        bar: i32,
        abc: String
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
    #[serde(rename = "renamed")]
    struct Bar {
        bar: i32
    }

    #[test]
    fn parse_renamed_json_struct() {
        match <Bar as ToParams>::Params::from_str("fields[renamed]=bar") {
            Ok(result) => assert_eq!(true, result.fields.bar),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_renamed_json_struct_fails_on_original_name() {
        match <Bar as ToParams>::Params::from_str("fields[bar]=bar") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::UnImplementedError, e)
        }
    }

    #[test]
    fn parse_present_field() {
        match <Foo as ToParams>::Params::from_str("fields[foo]=bar") {
            Ok(result) => assert_eq!(true, result.fields.bar),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_field_that_is_not_present() {
        match <Foo as ToParams>::Params::from_str("") {
            Ok(result) => assert_eq!(false, result.fields.bar),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_fields_fails_if_query_param_is_not_valid() {
        match <Foo as ToParams>::Params::from_str("fields=body=foo") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::InvalidParam("fields=body=foo".to_string()), e)
        }
    }

    #[test]
    fn parse_fields_fails_if_fields_value_is_empty() {
        match <Foo as ToParams>::Params::from_str("fields[foo]=") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::InvalidValue("Fields for foo are empty".to_string()), e)
        }
    }

    #[test]
    fn parse_fields_fails_if_field_value_contains_field_that_does_not_exist() {
        match <Foo as ToParams>::Params::from_str("fields[foo]=non_existent") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::InvalidValue("Invalid field: non_existent".to_string()), e)
        }
    }

    #[test]
    fn parse_single_field_fails_if_field_doesnt_contain_left_bracket() {
        match <Foo as ToParams>::Params::from_str("fieldsarticles]=title") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::InvalidKeyParam("articles]".to_string()), e)
        }
    }

    #[test]
    fn parse_single_field_fails_if_field_does_not_contain_right_bracket() {
        match <Foo as ToParams>::Params::from_str("fields[articles=title") {
            Ok(_) => assert!(false, "expected error but no error happened!"),
            Err(e) => assert_eq!(QueryStringParseError::InvalidKeyParam("[articles".to_string()), e)
        }
    }

    #[test]
    fn parse_sort_field() {
        match <Foo as ToParams>::Params::from_str("sort=bar") {
            Ok(result) => assert_eq!(Some(&bar(Asc)), result.sort_fields.first()),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_descending_sort_field() {
        match <Foo as ToParams>::Params::from_str("sort=-bar") {
            Ok(result) => assert_eq!(Some(&bar(Desc)), result.sort_fields.first()),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_multiple_sort_fields() {
        match <Foo as ToParams>::Params::from_str("sort=-bar,abc") {
            Ok(result) => {
                let expected = vec![bar(Desc), abc(Asc)];
                assert_eq!(expected, result.sort_fields)
            },
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }

    #[test]
    fn parse_null_sort_field() {
        match <Foo as ToParams>::Params::from_str("") {
            Ok(result) => assert_eq!(None, result.sort_fields.first()),
            Err(e) => assert!(false, format!("unexpected error!, {:?}", e))
        }
    }
}
