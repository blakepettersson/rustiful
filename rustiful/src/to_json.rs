use serde::ser::Serialize;

pub trait ToJson {
    type Attrs: Clone + Serialize;

    fn id(&self) -> String;

    fn type_name(&self) -> String;
}
