use std::collections::HashMap;

pub trait JsonApiResource {
    type Params;
}

pub trait TypedParams {
    type SortField;
    type FilterField;
    fn sort(&mut self) -> &mut Vec<Self::SortField>;
    fn filter(&mut self) -> &mut Vec<Self::FilterField>;
    fn query_params(&mut self) -> &mut HashMap<String, String>;
}