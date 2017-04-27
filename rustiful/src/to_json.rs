use id::JsonApiId;
use serde::ser::Serialize;

pub trait ToJson {
    type Attrs: Clone + Serialize;
    type Resource: Clone + Serialize;

    fn id(&self) -> JsonApiId;

    fn type_name(&self) -> String;
}
