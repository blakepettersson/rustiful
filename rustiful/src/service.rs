extern crate serde;

use FromRequest;
use data::JsonApiData;
use resource::JsonApiResource;
use status::Status;
use std;
use to_json::ToJson;

pub trait JsonGet where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn find(id: Self::JsonApiIdType,
            params: &Self::Params,
            ctx: Self::Context)
            -> Result<Option<JsonApiData<Self::Attrs>>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPost where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn create(json: JsonApiData<Self::Attrs>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self::Attrs>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonPatch where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn update(id: Self::JsonApiIdType,
              json: JsonApiData<Self::Attrs>,
              params: &Self::Params,
              ctx: Self::Context)
              -> Result<JsonApiData<Self::Attrs>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonIndex where Self: JsonApiResource + ToJson
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn find_all(params: &Self::Params,
                ctx: Self::Context)
                -> Result<Vec<JsonApiData<Self::Attrs>>, Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}

pub trait JsonDelete where Self: JsonApiResource
{
    type Error: std::error::Error + Send;
    type Context: FromRequest;

    fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), Self::Error>
        where Status: for<'b> From<&'b Self::Error>;
}
