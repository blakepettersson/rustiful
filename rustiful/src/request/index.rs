use super::Status;
use array::JsonApiArray;
use data::JsonApiData;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use params::TypedParams;
use service::JsonIndex;
use sort_order::SortOrder;
use to_json::ToJson;
use try_from::TryFrom;

autoimpl! {
    pub trait FromIndex<'a, T> where
        T: ToJson + JsonIndex,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: for<'b> From<(T, &'b T::Params)>,
        T::Params: TryFrom<(&'a str, Vec<&'a str>, T::Params), Error = QueryStringParseError>,
        T::Params: TryFrom<(&'a str, SortOrder, T::Params), Error = QueryStringParseError>,
        T::Params: TypedParams<T::SortField, T::FilterField> + Default
    {
        fn get(query: &'a str, ctx: T::Context)
        -> Result<JsonApiArray<JsonApiData<T::Attrs>>, RequestError<T::Error>> {
            match T::from_str(query) {
                Ok(params) => {
                    match T::find_all(&params, ctx) {
                        Ok(result) => {
                            let data: Vec<JsonApiData<T::Attrs>> = result.into_iter()
                                .map(|e| (e, &params).into())
                                .collect();
                            Ok(JsonApiArray::<_> { data: data })
                        },
                        Err(e) => Err(RequestError::RepositoryError(RepositoryError::new(e)))
                    }
                },
                Err(e) => Err(RequestError::QueryStringParseError(e))
            }
        }
    }
}