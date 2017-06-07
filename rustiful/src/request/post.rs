use super::Status;
use data::JsonApiData;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonPost;
use sort_order::SortOrder;
use std::error::Error;
use std::str::FromStr;
use try_from::TryFrom;

/// This is a utility function that calls `T::create()` and returns `JsonApiObject<T::Attrs>` if
/// successful.
///
pub fn post<'a, T>(query: &'a str,
                   json: JsonApiData<T::Attrs>,
                   ctx: T::Context)
                   -> Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>>
    where T: JsonPost,
          Status: for<'b> From<&'b T::Error>,
          T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
          T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>,
          <T::JsonApiIdType as FromStr>::Err: Error
{
    let params = T::from_str(query)
        .map_err(|e| RequestError::QueryStringParseError(e))?;
    let result = T::create(json, &params, ctx)
        .map_err(|e| RequestError::RepositoryError(RepositoryError::new(e)))?;
    Ok(JsonApiObject::<_> { data: result })
}
