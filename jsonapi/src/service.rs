extern crate serde;

use std;
use params::JsonApiResource;

pub trait JsonApiService {
    type Error: std::error::Error;
    type T: JsonApiResource;

    fn find(&self, id: <Self::T as JsonApiResource>::JsonApiIdType, params: &<Self::T as JsonApiResource>::Params) -> Result<Option<Self::T>, Self::Error>;

    fn find_all(&self, params: &<Self::T as JsonApiResource>::Params) -> Result<Vec<Self::T>, Self::Error>;

    fn save(&self, record: Self::T) -> Result<Self::T, Self::Error>;

    fn delete(&self, id: <Self::T as JsonApiResource>::JsonApiIdType) -> Result<(), Self::Error>;
}

pub trait ToRequest<T> where T: JsonApiService {}

