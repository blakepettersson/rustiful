use super::Status;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonGet;
use params::SortOrder;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

/// This is a utility function that calls `T::find()` and returns `JsonApiObject<T::Attrs>` if
/// successful.
///
pub fn get<'a, T>(id: T::JsonApiIdType,
                  query: &'a str,
                  ctx: T::Context)
                  -> Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>>
    where T: ToJson + JsonGet,
          <T::JsonApiIdType as FromStr>::Err: Error,
          Status: for<'b> From<&'b T::Error>,
          T::SortField: for<'b> TryFrom<(&'b str, SortOrder), Error = QueryStringParseError>,
          T::FilterField: for<'b> TryFrom<(&'b str, Vec<&'b str>), Error = QueryStringParseError>
{
    let params = T::Params::from_str(query).map_err(|e| RequestError::QueryStringParseError(e))?;
    let result = T::find(id, &params, ctx)
        .map_err(|e| RequestError::RepositoryError(RepositoryError::new(e)))?;
    let data = result.ok_or(RequestError::NotFound)?;
    Ok(JsonApiObject::<_> { data: data })
}
