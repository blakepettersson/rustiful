use serde::{Deserialize, Deserializer};

/// This function is used to deserialize a JSON property to a nested Option.
///
/// If the property is explicitly set to `null` in the JSON string, this will return `Some(None)`.
/// If the property is not given in the JSON string, this will instead return `None`, otherwise this
/// will return `Some(Some(T))`.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate serde_derive;
///
/// extern crate serde;
/// extern crate serde_json;
/// extern crate rustiful;
///
/// #[derive(Deserialize)]
/// struct Test {
///    id: String,
///    #[serde(default, deserialize_with = "rustiful::json_option::some_option")]
///    body: Option<Option<String>>
/// }
///
/// fn main() {
///    let body_blank: Test = serde_json::from_str(r#"{ "id": "1" }"#.as_ref()).unwrap();
///    let body_present: Test = serde_json::from_str(r#"{ "id": "1", "body": "funky" }"#.as_ref()).unwrap();
///    let body_null: Test = serde_json::from_str(r#"{ "id": "1", "body": null }"#.as_ref()).unwrap();
///
///    assert_eq!(None, body_blank.body);
///    assert_eq!(Some(Some("funky".to_string())), body_present.body);
///    assert_eq!(Some(None), body_null.body);
///}
/// ```
pub fn some_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    Option::<T>::deserialize(deserializer).map(Some)
}
