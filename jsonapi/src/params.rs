use std::collections::HashMap;

pub trait JsonApiResource {
    type Params;
}

pub trait TypedParams<SortField, FilterField> {
    fn sort(&mut self) -> &mut Vec<SortField>;
    fn filter(&mut self) -> &mut Vec<FilterField>;
    fn query_params(&mut self) -> &mut HashMap<String, String>;
}