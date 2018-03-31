use resources::diesel_resource::Test;
use resources::simple_resources::{Bar, Foo};
use rustiful::*;
use std::convert::TryInto;

#[test]
fn test_into_conversions_with_int_id() {
    let test = Foo {
        bar: 1,
        foo: 2,
        abc: "abc".to_string()
    };

    let expected_id = test.bar.to_string();
    let result: JsonApiData<Foo> = test.into_json(&Default::default());
    assert_eq!(expected_id, result.id.expect("unexpected None on id!"));
    assert_eq!(2, result.attributes.foo.expect("unexpected None on foo!"));
    assert_eq!(
        "abc".to_string(),
        result.attributes.abc.expect("unexpected None on abc!")
    );
}

#[test]
fn test_into_conversions_with_string_id() {
    let test = Bar {
        id: "test".to_string(),
        bar: 1
    };

    let expected_id = test.id.clone();
    let result: JsonApiData<Bar> = test.into_json(&Default::default());
    assert_eq!(expected_id, result.id.expect("unexpected None on id!"));
    assert_eq!(1, result.attributes.bar.expect("unexpected None on bar!"));
}

#[test]
fn test_setting_of_id_in_try_from() {
    let json_attrs = <Test as ToJson>::Attrs::new(Some("3".to_string()), None, None);
    let json = JsonApiData::new(Some("1".to_string()), json_attrs);
    let test = Test {
        id: "1".to_string(),
        title: "foo".to_string(),
        body: None,
        published: false
    };

    let expected_id = test.id.clone();
    let result: Test = (test, json).try_into().unwrap();
    assert_eq!(expected_id, result.id)
}
