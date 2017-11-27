use serde::de::Deserializer;
use serde::de::Error;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::ser::Serializer;
use std::fmt;
use std::marker::PhantomData;
use std::str;
use to_json::ToJson;

/// Serialises a `ToJson::TYPE_NAME` as a type property
///
/// # Example
///
/// Given a resource that implements `ToJson` (this is automatically implemented when
/// deriving `JsonApi`), such as the one below:
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// The `_type` field will be converted into a string value when serialising to JSON.
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # extern crate serde_json;
/// #
/// # use rustiful::ToJson;
/// # use rustiful::IntoJson;
/// # use serde_json::Value;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// # struct MyResource {
/// #     id: String,
/// #     foo: bool,
/// #     bar: String
/// # }
/// #
/// # fn main() {
/// let resource = MyResource {
///     id: "foo".to_string(),
///     foo: true,
///     bar: "abc".to_string()
/// };
/// let json = serde_json::to_string(&resource.into_json(&Default::default())).unwrap();
/// let v: Value = serde_json::from_str(&json).unwrap();
/// assert_eq!(v["type"], MyResource::TYPE_NAME)
/// # }
///
/// ```
pub fn serialize<S, T>(_: &PhantomData<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToJson
{
    serializer.serialize_str(T::TYPE_NAME)
}

/// Deserialises a type property as a `PhantomData`. If there is a mismatch between a type name
/// and `T:TYPE_NAME` an error will be returned.
///
/// # Example
///
/// Given a resource that implements `ToJson` (this is automatically implemented when
/// deriving `JsonApi`), such as the one below:
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
/// #
/// # fn main() {
/// # }
/// ```
///
/// The `type` field will be converted into a `PhantomData` type on a successful deserialisation.
///
/// ```
/// # extern crate rustiful;
/// #
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # #[macro_use]
/// # extern crate rustiful_derive;
/// #
/// # extern crate serde_json;
/// #
/// # use rustiful::ToJson;
/// # use rustiful::JsonApiData;
/// # use rustiful::JsonApiContainer;
/// # use rustiful::IntoJson;
/// # use serde_json::Value;
/// # use std::error::Error;
/// #
/// #[derive(Debug, PartialEq, Eq, Clone, JsonApi, Default)]
/// # struct MyResource {
/// #     id: String,
/// #     foo: bool,
/// #     bar: String
/// # }
/// #
/// # fn main() {
/// let valid_data = r#"{
///                    "id": "foo",
///                    "type": "my-resources",
///                    "attributes": {
///                       "bar": "foo"
///                    }
///                }"#;
/// // Should deserialise successfully
/// let _: JsonApiData<MyResource> = serde_json::from_str(&valid_data).unwrap();
///
/// let invalid_type_name = r#"{
///                                 "id": "foo",
///                                 "type": "something-invalid",
///                                 "attributes": {
///                                     "bar": "foo"
///                                 }
///                             }"#;
/// // Should fail to deserialise
/// let err: Result<JsonApiData<MyResource>, _> = serde_json::from_str(&invalid_type_name);
/// match err {
///     Ok(_) => { assert!(false, "Unexpected deserialisation success!") },
///     Err(e) => assert!(format!("{}", e).contains("Invalid type name 'something-invalid'"))
/// }
/// # }
///
/// ```
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<PhantomData<T>, D::Error>
where
    T: ToJson,
    D: Deserializer<'de>
{
    let value = deserializer.deserialize_string(StringVisitor)?;
    if value == T::TYPE_NAME {
        Ok(PhantomData)
    } else {
        Err(D::Error::custom(format!("Invalid type name '{}'", value)))
    }
}

struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<String, E>
    where
        E: Error
    {
        Ok(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<String, E>
    where
        E: Error
    {
        Ok(v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<String, E>
    where
        E: Error
    {
        match str::from_utf8(v) {
            Ok(s) => Ok(s.to_owned()),
            Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self))
        }
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<String, E>
    where
        E: Error
    {
        match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(e) => Err(Error::invalid_value(
                Unexpected::Bytes(&e.into_bytes()),
                &self
            ))
        }
    }
}
