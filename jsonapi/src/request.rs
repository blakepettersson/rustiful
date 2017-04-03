use std::error::Error;
use std::str::FromStr;
use data::JsonApiData;
use params::JsonApiResource;
use queryspec::ToJson;
use errors::RequestError;
use object::JsonApiObject;
use array::JsonApiArray;
use service::JsonGet;
use service::JsonIndex;
use params::TypedParams;
use service::JsonDelete;
use try_from::TryFrom;
use sort_order::SortOrder;
use query_string::QueryString;
use queryspec::QueryStringParseError;
use errors::RepositoryError;

pub trait FromGet<'a, T> where
    T: ToJson,
    T: JsonGet,
    T: JsonApiResource,
    T::JsonApiIdType: FromStr,
    <T as JsonGet>::Error : 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TypedParams<SortField = <T as JsonApiResource>::SortField, FilterField = <T as JsonApiResource>::FilterField> + Default,
    <T::JsonApiIdType as FromStr>::Err: Send + Error,
    <T::JsonApiIdType as FromStr>::Err: 'static,
{
    fn get(id: &'a str, query: &'a str, ctx: T::Context) -> Result<JsonApiObject<JsonApiData<<T as ToJson>::Attrs>>, RepositoryError> {
        match <T as JsonApiResource>::from_str(query) {
            Ok(params) => {
                match <<T as JsonApiResource>::JsonApiIdType>::from_str(id) {
                    Ok(typed_id) => {
                        match <T as JsonGet>::find(typed_id, &params, ctx) {
                            Ok(obj) => {
                                let data: Option<JsonApiData<<T as ToJson>::Attrs>> = obj.map(|obj| (obj, &params).into());
                                let res = data.ok_or(RequestError::NotFound)?;
                                Ok(JsonApiObject::<_> { data: res })
                            },
                            Err(e) => Err(RepositoryError { error: Box::new(e) })
                        }
                    },
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}

impl <'a, T> FromGet<'a, T> for T where
        T: ToJson,
        T: JsonGet,
        T: JsonApiResource,
        T::JsonApiIdType: FromStr,
        <T as JsonGet>::Error : 'static,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TypedParams<SortField = <T as JsonApiResource>::SortField, FilterField = <T as JsonApiResource>::FilterField> + Default,
        T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
        <T::JsonApiIdType as FromStr>::Err: Send + Error,
        <T::JsonApiIdType as FromStr>::Err: 'static {}

pub trait FromIndex<'a, T> where
    T: ToJson,
    T: JsonIndex,
    T: JsonApiResource,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TypedParams<SortField = <T as JsonApiResource>::SortField, FilterField = <T as JsonApiResource>::FilterField> + Default,
    <T as JsonIndex>::Error: Send,
    <T as JsonIndex>::Error : 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
{
    fn get(query: &'a str, ctx: T::Context) -> Result<JsonApiArray<JsonApiData<<T as ToJson>::Attrs>>, RepositoryError> {
        match <T as JsonApiResource>::from_str(query) {
            Ok(params) => {
                let params:<T as JsonApiResource>::Params = params.into();

                match <T as JsonIndex>::find(&params, ctx) {
                    Ok(result) => {
                        let data: Vec<JsonApiData<<T as ToJson>::Attrs>> = result.into_iter()
                            .map(|e| (e, &params).into())
                            .collect();
                        Ok(JsonApiArray::<_> { data: data })
                    },
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}

impl <'a, T> FromIndex<'a, T> for T where
        T: ToJson,
        T: JsonIndex,
        T: JsonApiResource,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TypedParams<SortField = <T as JsonApiResource>::SortField, FilterField = <T as JsonApiResource>::FilterField> + Default,
        <T as JsonIndex>::Error: Send,
        <T as JsonIndex>::Error : 'static,
        T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>
{
}

pub trait FromDelete<'a, T> where
    T: ToJson + JsonDelete + JsonApiResource,
    T::JsonApiIdType: FromStr,
    <T as JsonDelete>::Error : Send + 'static,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn delete(id: &'a str, ctx: T::Context) -> Result<(), RepositoryError> {
        match <<T as JsonApiResource>::JsonApiIdType>::from_str(id) {
            Ok(typed_id) => {
                match <T as JsonDelete>::delete(typed_id, ctx) {
                    Ok(_) => { Ok(()) },
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}

impl <'a, T> FromDelete<'a, T> for T where
    T: ToJson + JsonDelete + JsonApiResource + QueryString<'a>,
    T::JsonApiIdType: FromStr,
    <T as JsonDelete>::Error : Send + 'static,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {}
