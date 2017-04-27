use super::Status;
use data::JsonApiData;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use params::TypedParams;
use service::JsonGet;
use sort_order::SortOrder;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

autoimpl! {
    pub trait FromGet<'a, T> where
        T: ToJson + JsonGet,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default,
        <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn get(id: &'a str, query: &'a str, ctx: T::Context)
        -> Result<JsonApiObject<T::Attrs>, RequestError<T::Error>> {
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
                                Err(e) => {
                                    Err(RequestError::RepositoryError(RepositoryError::new(e)))
                                }
                            }
                        },
                        Err(e) => Err(RequestError::IdParseError(Box::new(e)))
                    }
                },
                Err(e) => Err(RequestError::QueryStringParseError(e))
            }
        }
    }
}
