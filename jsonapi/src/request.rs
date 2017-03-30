use std::error::Error;
use std::str::FromStr;
use data::JsonApiData;
use params::Params;
use params::TypedParams;
use params::JsonApiResource;
use try_from::TryFrom;
use queryspec::ToJson;
use errors::RequestError;
use object::JsonApiObject;
use array::JsonApiArray;
use sort_order::SortOrder;
use service::JsonApiService;
use query_string::QueryString;
use queryspec::QueryStringParseError;
use errors::RepositoryError;

pub trait FromRequest<'a, R, P, S, F>
    where Self: JsonApiService<T = R>,
          <Self as JsonApiService>::Error: 'static,
          P: Params,
          P: Default,
          P: TypedParams<SortField = S, FilterField = F>,
          P: for<'b> TryFrom<(&'b str, SortOrder, P), Err = QueryStringParseError>,
          P: for<'b> TryFrom<(&'b str, Vec<&'b str>, P), Err = QueryStringParseError>,
          R: ToJson,
          R: for<'b> QueryString<'b, Params = P, SortField = S, FilterField = F>,
          R: JsonApiResource<Params = P>,
          <R as ToJson>::Attrs: for<'b> From<(R, &'b P)>,
          <R as JsonApiResource>::JsonApiIdType: FromStr,
          <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Send,
          <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: Error,
          <<R as JsonApiResource>::JsonApiIdType as FromStr>::Err: 'static,
{
    fn get(&self, id: &'a str, query: &'a str) -> Result<JsonApiObject<JsonApiData<<R as ToJson>::Attrs>>, RepositoryError> {
        match <R as QueryString>::from_str(query) {
            Ok(params) => {
                match <<R as JsonApiResource>::JsonApiIdType>::from_str(id) {
                    Ok(typed_id) => {
                        match self.find(typed_id, &params) {
                            Ok(result) => {
                                let data: Option<JsonApiData<<R as ToJson>::Attrs>> = result.map(|obj| (obj, &params).into());
                                let res = data.ok_or(RequestError::NotFound)?;
                                Ok(JsonApiObject::<_> { data: res })
                            },
                            Err(e) => Err(RepositoryError { error: Box::new(e) })
                        }
                    },
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }

    fn index(&self, query: &'a str) -> Result<JsonApiArray<JsonApiData<<R as ToJson>::Attrs>>, RepositoryError> {
        match <R as QueryString>::from_str(query) {
            Ok(params) => {
                match self.find_all(&params) {
                    Ok(result) => {
                        let data: Vec<JsonApiData<<R as ToJson>::Attrs>> = result.into_iter()
                            .map(|e| (e, &params).into())
                            .collect();
                        Ok(JsonApiArray::<_> { data: data })
                    },
                    Err(e) => Err(RepositoryError { error: Box::new(e) })
                }
            },
            Err(e) => Err(RepositoryError { error: Box::new(e) })
        }
    }
}
