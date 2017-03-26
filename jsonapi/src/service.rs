extern crate serde;

use std;
use queryspec::ToJson;
use params::JsonApiResource;

pub trait JsonApiService
{
    type Error: std::error::Error;
    type T: JsonApiResource + ToJson;

    fn find(&self, id: &str, params: &<Self::T as JsonApiResource>::Params) -> Result<Option<Self::T>, Self::Error>;

    fn find_all(&self, params: &<Self::T as JsonApiResource>::Params) -> Result<Vec<Self::T>, Self::Error>;

    fn save(&self, record: Self::T) -> Result<Self::T, Self::Error>;

    fn delete(&self, id: &str) -> Result<(), Self::Error>;
}

pub trait ToRequest<T> where T: JsonApiService {}

