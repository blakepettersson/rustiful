extern crate serde;

use std;
use params::JsonApiResource;

pub trait JsonApiService {
    type Error: std::error::Error + Send;
    type T: JsonApiResource;

    fn find(&self,
            id: <Self::T as JsonApiResource>::JsonApiIdType,
            params: &<Self::T as JsonApiResource>::Params)
            -> Result<Option<Self::T>, Self::Error>;

    fn find_all(&self,
                params: &<Self::T as JsonApiResource>::Params)
                -> Result<Vec<Self::T>, Self::Error>;

    fn save(&self, record: Self::T) -> Result<Self::T, Self::Error>;

    fn delete(&self, id: <Self::T as JsonApiResource>::JsonApiIdType) -> Result<(), Self::Error>;
}

pub trait JsonGet
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<Self>, Self::Error>;
}

pub trait JsonIndex
    where Self: JsonApiResource
{
    type Error: std::error::Error;
    type Context: Default;

    fn find(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error>;
}

pub trait JsonDelete
    where Self: JsonApiResource
{
    type Error: std::error::Error;
    type Context: Default;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error>;
}
