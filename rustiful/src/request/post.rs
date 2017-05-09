use super::Status;
use data::JsonApiData;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use params::JsonApiParams;
use service::JsonPost;
use std::error::Error;
use std::str::FromStr;

autoimpl! {
    pub trait FromPost<'a, T>
        where T: JsonPost,
              Status: for<'b> From<&'b T::Error>,
              T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>,
              <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn create(json: JsonApiData<T::Attrs>, ctx: T::Context) ->
        Result<JsonApiObject<T::Attrs>, RequestError<T::Error, T::JsonApiIdType>> {
            match <T as JsonPost>::create(json, ctx) {
                Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
                Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
            }
        }
    }
}
