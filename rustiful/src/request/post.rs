use data::JsonApiData;
use object::JsonApiObject;
use errors::RequestError;
use errors::RepositoryError;
use service::JsonPost;
use params::JsonApiResource;
use to_json::ToJson;
use super::Status;

autoimpl! {
    pub trait FromPost<'a, T>
        where T: ToJson + JsonPost + JsonApiResource,
              Status: for<'b> From<&'b T::Error>,
              T::Attrs: for<'b> From<(T, &'b T::Params)>
    {
        fn create(json: T::Resource, ctx: T::Context) -> Result<JsonApiObject<JsonApiData<T::Attrs>>, RequestError<T::Error>> {
            match <T as JsonPost>::create(json, ctx) {
                Ok(result) => Ok(JsonApiObject::<_> { data: result.into() }),
                Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
            }
        }
    }
}