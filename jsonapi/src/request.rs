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
use queryspec::QueryStringParseError;
use errors::RepositoryError;

pub trait FromGet<'a, T> where
    T: ToJson + JsonGet + JsonApiResource,
    T::JsonApiIdType: FromStr,
    <T as JsonGet>::Error : 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TypedParams<<T as JsonApiResource>::SortField, <T as JsonApiResource>::FilterField> + Default,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
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
    T: ToJson + JsonGet + JsonApiResource,
    T::JsonApiIdType: FromStr,
    <T as JsonGet>::Error : 'static,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TypedParams<<T as JsonApiResource>::SortField, <T as JsonApiResource>::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {}

pub trait FromIndex<'a, T> where
    T: ToJson + JsonIndex + JsonApiResource,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
    <T as JsonApiResource>::Params: TypedParams<<T as JsonApiResource>::SortField, <T as JsonApiResource>::FilterField> + Default,
    <T as JsonIndex>::Error: Send + 'static,
    T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>,
{
    fn get(query: &'a str, ctx: T::Context) -> Result<JsonApiArray<JsonApiData<<T as ToJson>::Attrs>>, RepositoryError> {
        match <T as JsonApiResource>::from_str(query) {
            Ok(params) => {
                let params:<T as JsonApiResource>::Params = params.into();

                match <T as JsonIndex>::find_all(&params, ctx) {
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
        T: ToJson + JsonIndex + JsonApiResource,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, Vec<&'a str>, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TryFrom<(&'a str, SortOrder, <T as JsonApiResource>::Params), Err = QueryStringParseError>,
        <T as JsonApiResource>::Params: TypedParams<<T as JsonApiResource>::SortField, <T as JsonApiResource>::FilterField> + Default,
        <T as JsonIndex>::Error: Send + 'static,
        T::Attrs: for<'b> From<(T, &'b <T as JsonApiResource>::Params)>
{
}

pub trait FromDelete<'a, T>
    where T: ToJson + JsonDelete + JsonApiResource,
          T::JsonApiIdType: FromStr,
          <T as JsonDelete>::Error: Send + 'static,
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

impl<'a, T> FromDelete<'a, T> for T
    where T: ToJson + JsonDelete + JsonApiResource,
          T::JsonApiIdType: FromStr,
          <T as JsonDelete>::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
}
