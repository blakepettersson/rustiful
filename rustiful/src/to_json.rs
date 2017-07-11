
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub trait ToJson {
    type Attrs: Clone + Serialize + DeserializeOwned;

    fn id(&self) -> String;

    fn type_name(&self) -> String;
}
