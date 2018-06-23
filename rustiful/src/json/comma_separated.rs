use serde::de::{Deserialize, Deserializer, Visitor};
use std::iter::FromIterator;
use std::str::FromStr;
use std::fmt::Display;
use std::fmt;
use std::marker::PhantomData;
use serde::de::Error;

pub fn deserialize<'de, V, T, E, D>(deserializer: D) -> Result<V, D::Error>
    where V: FromIterator<T>,
          T: FromStr<Err = E>,
          E: Display,
          D: Deserializer<'de>
{
    struct Ret<V, T, E>(V, PhantomData<T>, PhantomData<E>);
    struct CommaSeparated<V, T, E>(PhantomData<V>, PhantomData<T>, PhantomData<E>);

    impl<'de, V, T, E> Visitor<'de> for CommaSeparated<V, T, E>
        where V: FromIterator<T>,
              T: FromStr<Err = E>,
              E: Display
    {
        type Value = Ret<V, T, E>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<F>(self, s: &str) -> Result<Self::Value, F>
            where F: Error
        {
            let iter = s.split(",").filter(|&f| !f.is_empty()).map(FromStr::from_str);
            match FromIterator::from_iter(iter) {
                Ok(v) => Ok(Ret(v, PhantomData, PhantomData)),
                Err(e) => Err(Error::custom(e.to_string())),
            }
        }
    }

    impl<'de, V, T, E> Deserialize<'de> for Ret<V, T, E>
        where V: FromIterator<T>,
              T: FromStr<Err = E>,
              E: Display
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: Deserializer<'de>
        {
            let visitor = CommaSeparated(PhantomData, PhantomData, PhantomData);
            deserializer.deserialize_str(visitor)
        }
    }

    Deserialize::deserialize(deserializer).map(|ret: Ret<V, T, E>| ret.0)
}
