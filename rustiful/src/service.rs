extern crate serde;

use FromRequest;
use data::JsonApiData;
use params::JsonApiParams;
use params::JsonApiResource;
use status::Status;
use std;
use to_json::ToJson;

pub trait JsonGet
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn find(id: Self::JsonApiIdType,
            params: &JsonApiParams<Self::FilterField, Self::SortField>,
            ctx: Self::Context)
            -> Result<Option<Self>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPost
    where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn create(json: JsonApiData<Self::Attrs>, ctx: Self::Context) -> Result<Self, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPatch
    where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn update(id: Self::JsonApiIdType,
              json: JsonApiData<Self::Attrs>,
              ctx: Self::Context)
              -> Result<Self, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonIndex
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn find_all(params: &JsonApiParams<Self::FilterField, Self::SortField>,
                ctx: Self::Context)
                -> Result<Vec<Self>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonDelete
    where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}
