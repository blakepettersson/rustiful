use super::Status;
use data::JsonApiData;
use errors::RepositoryError;
use errors::RequestError;
use object::JsonApiObject;
use service::JsonPost;

autoimpl! {
    pub trait FromPost<'a, T>
        where T: JsonPost,
              Status: for<'b> From<&'b T::Error>,
              T::Attrs: for<'b> From<(T, &'b T::Params)>
    {
        fn create(json: T::Resource, ctx: T::Context) ->
            Result<JsonApiObject<JsonApiData<T::Attrs>>, RequestError<T::Error>> {
            match <T as JsonPost>::create(json, ctx) {
                Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
                Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
            }
        }
    }
}
