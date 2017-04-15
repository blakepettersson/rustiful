use super::Status;
use data::JsonApiData;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonPatch;
use std::error::Error;
use std::str::FromStr;

autoimpl! {
    pub trait FromPatch<'a, T>
        where T: JsonPatch,
              Status: for<'b> From<&'b T::Error>,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn patch(id: &'a str, json: T::Resource, ctx: T::Context)
        -> Result<JsonApiObject<JsonApiData<T::Attrs>>, RequestError<T::Error>> {
            match <T::JsonApiIdType>::from_str(id) {
                Ok(typed_id) => {
                    match <T as JsonPatch>::update(typed_id, json, ctx) {
                        Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
                        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                    }
                },
                Err(e) => Err(RequestError::IdParseError(Box::new(e)))
            }
        }
    }
}
