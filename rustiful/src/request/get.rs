use super::Status;
use data::JsonApiData;
use errors::IdParseError;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use params::JsonApiParams;
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
        T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn get(id: &'a str, query: &'a str, ctx: T::Context)
        -> Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>> {
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
                        Err(e) => Err(RequestError::IdParseError(IdParseError(e)))
                    }
                },
                Err(e) => Err(RequestError::QueryStringParseError(e))
            }
        }
    }
}
