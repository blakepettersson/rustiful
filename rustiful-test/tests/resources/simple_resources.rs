#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
pub struct Foo {
    #[JsonApiId]
    pub bar: i32,
    pub foo: i32,
    pub abc: String,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonApi)]
#[serde(rename = "renamed")]
pub struct Bar {
    pub id: String,
    pub bar: i32,
}
