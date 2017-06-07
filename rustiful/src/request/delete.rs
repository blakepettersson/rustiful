use super::Status;
use errors::RepositoryError;
use errors::RequestError;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;

/// This is a utility function that calls `T::delete()` and returns `()` if successful.
///
pub fn delete<'a, T>(id: T::JsonApiIdType, ctx: T::Context)
    -> Result<(), RequestError<T::Error, T::JsonApiIdType>>
    where
        T: JsonDelete,
        Status: for<'b> From<&'b T::Error>,
        <T::JsonApiIdType as FromStr>::Err: Error {

    match T::delete(id, ctx) {
        Ok(_) => Ok(()),
        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
    }
}
