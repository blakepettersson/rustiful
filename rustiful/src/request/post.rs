use super::Status;
use data::JsonApiData;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonPost;
use std::error::Error;
use std::str::FromStr;
use try_from::TryFrom;
use sort_order::SortOrder;
use errors::QueryStringParseError;

autoimpl! {
    pub trait FromPost<'a, T>
        where T: JsonPost,
              Status: for<'b> From<&'b T::Error>,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
              <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn create(query: &'a str, json: JsonApiData<T::Attrs>, ctx: T::Context) ->
        Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>> {
            match T::from_str(query) {
                Ok(params) => {
                    match <T as JsonPost>::create(json, &params, ctx) {
                        Ok(result) => Ok(JsonApiObject::<_> { data: result }),
                        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                    }
                },
                Err(e) => Err(RequestError::QueryStringParseError(e))
            }
        }
    }
}
