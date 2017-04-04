use id::JsonApiId;
use serde::ser::Serialize;
use serde::de::Deserialize;

pub trait ToJson {
    type Attrs: Serialize + Deserialize;
    type Resource: Serialize + Deserialize;

    fn id(&self) -> JsonApiId;

    fn type_name(&self) -> String;
}

