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
use service::JsonPatch;
use service::JsonPost;
use try_from::TryFrom;
use sort_order::SortOrder;
use errors::RepositoryError;
use errors::QueryStringParseError;

type JsonApiArrayResult<T> = Result<JsonApiArray<JsonApiData<T>>, RepositoryError>;
type JsonApiSingleResult<T> = Result<JsonApiObject<JsonApiData<T>>, RepositoryError>;

pub trait FromGet<'a, T> where
    T: ToJson + JsonGet + JsonApiResource,
    T::JsonApiIdType: FromStr,
    T::Error : 'static,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
{
    fn get(id: &'a str, query: &'a str, ctx: T::Context) -> JsonApiSingleResult<T::Attrs> {
        match T::from_str(query) {
            Ok(params) => {
                match <T::JsonApiIdType>::from_str(id) {
                    Ok(typed_id) => {
                        match T::find(typed_id, &params, ctx) {
                            Ok(obj) => {
                                let data: Option<JsonApiData<T::Attrs>> = obj.map(|obj| {
                                    (obj, &params).into()
                                });
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
    T::Error : 'static,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
    <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static {}

pub trait FromIndex<'a, T> where
    T: ToJson + JsonIndex + JsonApiResource,
    T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
    T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
    T::Params: TypedParams<T::SortField, T::FilterField> + Default,
    T::Error: Send + 'static,
    T::Attrs: for<'b> From<(T, &'b T::Params)>,
{
    fn get(query: &'a str, ctx: T::Context) -> JsonApiArrayResult<T::Attrs> {
        match T::from_str(query) {
            Ok(params) => {
                match T::find_all(&params, ctx) {
                    Ok(result) => {
                        let data: Vec<JsonApiData<T::Attrs>> = result.into_iter()
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
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        T::Error: Send + 'static,
        T::Attrs: for<'b> From<(T, &'b T::Params)> {}

pub trait FromPost<'a, T>
    where T: ToJson + JsonPost + JsonApiResource,
          T::JsonApiIdType: FromStr,
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
          T::Attrs: for<'b> From<(T, &'b T::Params)>,
{
    fn create(json: T::Resource, ctx: T::Context) -> JsonApiSingleResult<T::Attrs> {
        match <T as JsonPost>::create(json, ctx) {
            Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}

impl<'a, T> FromPost<'a, T> for T
    where T: ToJson + JsonPost + JsonApiResource,
          T::JsonApiIdType: FromStr,
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
          T::Attrs: for<'b> From<(T, &'b T::Params)>
{
}

pub trait FromPatch<'a, T>
    where T: ToJson + JsonPatch + JsonApiResource,
          T::JsonApiIdType: FromStr,
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
          T::Attrs: for<'b> From<(T, &'b T::Params)>,
{
    fn patch(id: &'a str, json: T::Resource, ctx: T::Context) -> JsonApiSingleResult<T::Attrs> {
        match <T::JsonApiIdType>::from_str(id) {
            Ok(typed_id) => {
                match <T as JsonPatch>::update(typed_id, json, ctx) {
                    Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}

impl<'a, T> FromPatch<'a, T> for T
    where T: ToJson + JsonPatch + JsonApiResource,
          T::JsonApiIdType: FromStr,
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static,
          T::Attrs: for<'b> From<(T, &'b T::Params)>
{
}

pub trait FromDelete<'a, T>
    where T: ToJson + JsonDelete + JsonApiResource,
          T::JsonApiIdType: FromStr,
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
    fn delete(id: &'a str, ctx: T::Context) -> Result<(), RepositoryError> {
        match <T::JsonApiIdType>::from_str(id) {
            Ok(typed_id) => {
                match T::delete(typed_id, ctx) {
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
          T::Error: Send + 'static,
          <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
{
}
