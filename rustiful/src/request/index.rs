use super::Status;
use array::JsonApiArray;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use service::JsonIndex;
use sort_order::SortOrder;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

autoimpl! {
    pub trait FromIndex<'a, T> where
        T: ToJson + JsonIndex,
        Status: for<'b> From<&'b T::Error>,
        T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn get(query: &'a str, ctx: T::Context)
        -> Result<JsonApiArray<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>> {
            match T::from_str(query) {
                Ok(params) => {
                    match T::find_all(&params, ctx) {
                        Ok(result) => Ok(JsonApiArray { data: result }),
                        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                    }
                },
                Err(e) => Err(RequestError::QueryStringParseError(e))
            }
        }
    }
}
