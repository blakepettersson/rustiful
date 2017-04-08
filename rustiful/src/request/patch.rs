use data::JsonApiData;
use object::JsonApiObject;
use std::error::Error;
use std::str::FromStr;
use errors::RequestError;
use errors::RepositoryError;
use service::JsonPatch;
use params::JsonApiResource;
use to_json::ToJson;
use super::Status;

autoimpl! {
    pub trait FromPatch<'a, T>
        where T: ToJson + JsonPatch + JsonApiResource,
              Status: for<'b> From<&'b T::Error>,
              T::Attrs: for<'b> From<(T, &'b T::Params)>,
              <T::JsonApiIdType as FromStr>::Err: Send + Error + 'static
    {
        fn patch(id: &'a str, json: T::Resource, ctx: T::Context) -> Result<JsonApiObject<JsonApiData<T::Attrs>>, RequestError<T::Error>> {
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