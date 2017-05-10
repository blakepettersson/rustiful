use super::Status;
use array::JsonApiArray;
use data::JsonApiData;
use errors::QueryStringParseError;
use errors::RepositoryError;
use errors::RequestError;
use params::JsonApiParams;
use service::JsonIndex;
use sort_order::SortOrder;
use std::error::Error;
use std::str::FromStr;
use to_json::ToJson;
use try_from::TryFrom;

autoimpl! {
    pub trait FromIndex<'a, T> where
        T: ToJson + JsonIndex,
        Status: for<'b> From<&'b T::Error>,
        T::Attrs: for<'b> From<(T, &'b JsonApiParams<T::FilterField, T::SortField>)>,
        T::SortField: TryFrom<(&'a str, SortOrder), Error = QueryStringParseError>,
        T::FilterField: TryFrom<(&'a str, Vec<&'a str>), Error = QueryStringParseError>,
        <T::JsonApiIdType as FromStr>::Err: Error
    {
        fn get(query: &'a str, ctx: T::Context)
        -> Result<JsonApiArray<JsonApiData<T::Attrs>>, RequestError<T::Error, T::JsonApiIdType>> {
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
