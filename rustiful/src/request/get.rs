use super::Status;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use service::JsonGet;
use params::SortOrder;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;
use data::JsonApiData;
use container::JsonApiContainer;

/// This is a utility function that calls `T::find()` and returns `JsonApiObject<T::Attrs>` if
/// successful.
///
pub fn get<'a, T>(id: T::JsonApiIdType,
                  query: &'a str,
                  ctx: T::Context)
                  -> Result<JsonApiContainer<JsonApiData<T>>, RequestError<T::Error>>
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
    Ok(JsonApiContainer { data: data })
}
