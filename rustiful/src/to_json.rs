use id::JsonApiId;
use serde::de::Deserialize;
use serde::ser::Serialize;

pub trait ToJson {
    type Attrs: Clone + Serialize + Deserialize;
    type Resource: Clone + Serialize + Deserialize;

    fn id(&self) -> JsonApiId;

    fn type_name(&self) -> String;
}
