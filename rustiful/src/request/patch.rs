use super::Status;
use data::JsonApiData;
use errors::IdParseError;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonPatch;
use std::error::Error;
use std::str::FromStr;
use sort_order::SortOrder;
use try_from::TryFrom;
use errors::QueryStringParseError;

autoimpl! {
    pub trait FromPatch<'a, T>
        where T: JsonPatch,
              Status: for<'b> From<&'b T::Error>,
              T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
              T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
              <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn patch(id: &'a str, query: &'a str, json: JsonApiData<T::Attrs>, ctx: T::Context)
        -> Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>> {
            match <T::JsonApiIdType>::from_str(id) {
                Ok(typed_id) => {
                    match T::from_str(query) {
                        Ok(params) => {
                            match <T as JsonPatch>::update(typed_id, json, &params, ctx) {
                                Ok(result) => Ok(JsonApiObject::<_> { data: result }),
                                Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                            }
                        },
                        Err(e) => Err(RequestError::QueryStringParseError(e))
                    }
                },
                Err(e) => Err(RequestError::IdParseError(IdParseError(e)))
            }
        }
    }
}
