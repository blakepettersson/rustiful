extern crate serde;

use std;
use params::JsonApiResource;

//fn save(&self, record: Self::T) -> Result<Self::T, Self::Error>;

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

    fn find_all(params: &Self::Params, ctx: Self::Context) -> Result<Vec<Self>, Self::Error>;
}

pub trait JsonDelete
    where Self: JsonApiResource
{
    type Error: std::error::Error;
    type Context: Default;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error>;
}
