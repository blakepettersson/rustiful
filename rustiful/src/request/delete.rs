use super::Status;
use errors::RepositoryError;
use errors::RequestError;
use service::JsonDelete;
use std::error::Error;
use std::str::FromStr;

autoimpl! {
    pub trait FromDelete<'a, T>
        where T: JsonDelete,
              Status: for<'b> From<&'b T::Error>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn delete(id: &'a str, ctx: T::Context) -> Result<(), RequestError<T::Error>> {
            match <T::JsonApiIdType>::from_str(id) {
                Ok(typed_id) => {
                    match T::delete(typed_id, ctx) {
                        Ok(_) => { Ok(()) },
                        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                    }
                },
                Err(e) => Err(RequestError::IdParseError(Box::new(e)))
            }
        }
    }
}