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

/// This is a utility function that calls `T::find_all()` and returns `JsonApiObject<T::Attrs>` if
/// successful.
///
pub fn index<'a, T>(query: &'a str,
                    ctx: T::Context)
                    -> Result<JsonApiArray<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>>
    where T: ToJson + JsonIndex,
          <T::JsonApiIdType as FromStr>::Err: Error,
          Status: for<'b> From<&'b T::Error>,
          T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
          T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>
{
    let params = T::from_str(query)
        .map_err(|e| RequestError::QueryStringParseError(e))?;
    let result = T::find_all(&params, ctx)
        .map_err(|e| RequestError::RepositoryError(RepositoryError::new(e)))?;
    Ok(JsonApiArray::<_> { data: result })
}
