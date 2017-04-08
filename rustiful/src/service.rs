extern crate serde;

use std;
use to_json::ToJson;
use params::JsonApiResource;
use status::Status;


pub trait JsonGet
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn find(id: Self::JsonApiIdType, params: &Self::Params, ctx: Self::Context)
        -> Result<Option<Self>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPost
    where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn create(json: Self::Resource, ctx: Self::Context)
        -> Result<Self, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPatch
    where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn update(id: Self::JsonApiIdType, json: Self::Resource, ctx: Self::Context)
        -> Result<Self, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonIndex
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn find_all(params: &Self::Params, ctx: Self::Context)
        -> Result<Vec<Self>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonDelete
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: Default;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context)
        -> Result<(), Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}
