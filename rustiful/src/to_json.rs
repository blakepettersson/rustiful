use serde::ser::Serialize;
use serde::de::DeserializeOwned;

pub trait ToJson {
    type Attrs: Clone + Serialize + DeserializeOwned;

    fn id(&self) -> String;

    fn type_name(&self) -> String;
}
