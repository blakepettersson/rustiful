use data::JsonApiData;
use resource::JsonApiResource;
use std;
use to_json::ToJson;
use try_from::TryFrom;
use try_from::TryInto;

/// A trait containing framework-specific types.
///
/// This is currently used to set the `Status` type for `JsonGet`, `JsonPost`, `JsonIndex`,
/// `JsonPatch` and `JsonDelete`. There is a blanket impl in `iron/mod.rs` for all `T` implementing
/// `JsonApiResource`, which sets the `Status` to the Hyper `Status` type.
pub trait Handler {
    type Status: Send;
}

/// A trait for implementing GET `/{resource-name}/{id}` on a resource type.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate rustiful_derive;
///
/// use std::error::Error;
/// use std::fmt::Display;
/// use rustiful::JsonGet;
/// use rustiful::ToJson;
/// use rustiful::IntoJson;
/// use rustiful::JsonApiData;
/// use rustiful::JsonApiParams;
///
/// #[derive(Debug, PartialEq, Eq, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # #[derive(Debug, PartialEq, Eq)]
/// # struct MyError(String);
/// #
/// # impl Error for MyError {
/// #    fn description(&self) -> &str {
/// #        &self.0
/// #    }
/// # }
/// #
/// # impl Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
/// #        self.0.fmt(f)
/// #    }
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// impl JsonGet for MyResource {
///     type Error = MyError;
///     type Context = MyCtx;
///
///     fn find(id: Self::JsonApiIdType,
///         params: &Self::Params,
///         ctx: Self::Context)
///         -> Result<Option<JsonApiData<Self>>, (Self::Error, Self::Status)> {
///         let resource = MyResource {
///             id: "magic_id".to_string(),
///             foo: true,
///             bar: "hello".to_string()
///         };
///
///         if id == resource.id {
///             Ok(Some(resource.into_json(params)))
///         } else {
///             Ok(None)
///         }
///     }
/// }
///
/// fn main() {
///     let id = "magic_id".to_string();
///     let attrs = <<MyResource as ToJson>::Attrs>::new(Some(true), Some("hello".to_string()));
///     let resource = MyResource::find(id.clone(), &Default::default(), MyCtx {});
///     let expected = JsonApiData::new(Some(id), attrs);
///     assert_eq!(expected, resource.unwrap().unwrap());
///
///     assert_eq!(Ok(None), MyResource::find("foo".to_string(), &Default::default(), MyCtx {}));
/// }
/// ```
pub trait JsonGet: Handler
where
    Self: JsonApiResource + ToJson
{
    /// A user-defined error type
    type Error: std::error::Error + Send;
    /// A user-defined type
    type Context;

    /// Gets a resource.
    ///
    /// * `id` - The id of the resource to delete. `Self::JsonApiIdType` is the
    /// type of the field of `Self` which is either named `id` or which has the `JsonApiId`
    /// attribute. In other words, if `Self` has a field named `id` which is a `Uuid`, then
    /// `Self::JsonApiIdType` will be `Uuid`. The only requirement is that `Self::JsonApiIdType` is
    /// convertible to a String (i.e the type implements `FromStr`).
    /// * `params` - A type-safe reference of the params
    /// passed in the request. `Self::FilterField` and `Self::SortField` are automatically
    /// implemented when `JsonApi` is derived. See `JsonApiParams` for more info.
    /// * `ctx` - A user defined context type. This is used to instantiate the given
    /// type on each request. This type can be used for whatever you like, such as an auth token
    fn find(
        id: Self::JsonApiIdType,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<Option<JsonApiData<Self>>, (Self::Error, Self::Status)>;
}

/// A trait for implementing POST `/{resource-name}` on a resource type.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate rustiful_derive;
///
/// use std::error::Error;
/// use std::fmt::Display;
/// use rustiful::TryInto;
/// use rustiful::JsonPost;
/// use rustiful::ToJson;
/// use rustiful::IntoJson;
/// use rustiful::JsonApiData;
/// use rustiful::JsonApiParams;
/// use rustiful::iron::status::Status;
///
/// #[derive(Debug, PartialEq, Eq, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # #[derive(Debug, PartialEq, Eq)]
/// # struct MyError(String);
/// #
/// # impl Error for MyError {
/// #    fn description(&self) -> &str {
/// #        &self.0
/// #    }
/// # }
/// #
/// # impl Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
/// #        self.0.fmt(f)
/// #    }
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// impl JsonPost for MyResource {
///     type Error = MyError;
///     type Context = MyCtx;
///
///     fn create(json: JsonApiData<Self>,
///               params: &Self::Params,
///               ctx: Self::Context)
///             -> Result<JsonApiData<Self>, (Self::Error, Status)> {
///         if let Some(_) = json.id {
///             Err((MyError("invalid id!".to_string()), Status::BadRequest))
///         } else {
///             let mut resource: Self = json.try_into().map_err(|e| (MyError(e), Status::BadRequest))?;
///             resource.id = "created!".to_string();
///             Ok(resource.into_json(params))
///         }
///     }
/// }
///
/// fn main() {
///     let id = "some_id".to_string();
///     let attrs = <<MyResource as ToJson>::Attrs>::new(Some(true), Some("hello".to_string()));
///     let mut resource = JsonApiData::new(Some(id), attrs);
///
///     let err = Err((MyError("invalid id!".to_string()), Status::BadRequest));
///     assert_eq!(err, MyResource::create(resource.clone(), &Default::default(), MyCtx {}));
///
///     let mut expected = resource.clone();
///     expected.id = Some("created!".to_string());
///
///     resource.id = None;
///
///     assert_eq!(Ok(expected), MyResource::create(resource, &Default::default(), MyCtx {}));
/// }
/// ```
pub trait JsonPost: Handler
where
    Self: JsonApiResource + ToJson
{
    /// A user-defined error type
    type Error: std::error::Error + Send;

    /// A user-defined type
    type Context;

    /// Creates a resource.
    ///
    /// * `json` - The JSON document to create your record with. Convert it to
    /// `Result<Self, String>` using `json::try_into()` (a `TryFrom<JsonApiData<Self>>`
    /// impl is automatically generated when `JsonApi` is derived).
    /// It is also possible to use a client-generated id when creating a new resource; the id will
    /// be present in `json.id` if the client decides to generate an id. If the client doesn't
    /// generate an id `json.id` will be `None`, and it will be up to the implementation to generate
    /// a suitable id. If client-generated ids are not supported, an Error should be returned.
    /// * `params` - A type-safe reference of the params
    /// passed in the request. `Self::FilterField` and `Self::SortField` are automatically
    /// implemented when `JsonApi` is derived. See `JsonApiParams` for more info.
    /// * `ctx` - A user defined context type. This is used to instantiate the given
    /// type on each request. This type can be used for whatever you like, such as an auth token
    /// or a database connection.
    fn create(
        json: JsonApiData<Self>,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)>;
}

/// A trait for implementing PATCH `/{resource-name}/{id}` on a resource type.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate rustiful_derive;
///
/// use std::error::Error;
/// use std::fmt::Display;
/// use rustiful::JsonPatch;
/// use rustiful::ToJson;
/// use rustiful::IntoJson;
/// use rustiful::JsonApiData;
/// use rustiful::JsonApiParams;
/// use rustiful::iron::status::Status;
///
/// #[derive(Debug, PartialEq, Eq, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # #[derive(Debug, PartialEq, Eq)]
/// # struct MyError(String);
/// #
/// # impl Error for MyError {
/// #    fn description(&self) -> &str {
/// #        &self.0
/// #    }
/// # }
/// #
/// # impl Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
/// #        self.0.fmt(f)
/// #    }
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// impl JsonPatch for MyResource {
///     type Error = MyError;
///     type Context = MyCtx;
///
///     fn update(id: Self::JsonApiIdType,
///               json: JsonApiData<Self>,
///               params: &Self::Params,
///               ctx: Self::Context)
///               -> Result<JsonApiData<Self>, (Self::Error, Status)> {
///         let mut resource = MyResource {
///             id: "magic_id".to_string(),
///             foo: true,
///             bar: "hello".to_string()
///         };
///
///         if id == resource.id {
///             // The `patch` method will only overwrite fields that are explicitly sent in the
///             // JSON patch, i.e if the field has a value or is explicitly set to `null`. Fields
///             // that are omitted will not be updated.
///             let updated: MyResource = resource.patch(json).map_err(|e| (MyError(e), Status::BadRequest))?;
///             Ok(updated.into_json(params))
///         } else {
///             Err((MyError("Cannot patch resource!".to_string()), Status::BadRequest))
///         }
///     }
/// }
///
/// fn main() {
///     let id = "magic_id".to_string();
///     let attrs = <<MyResource as ToJson>::Attrs>::new(None, Some("updated".to_string()));
///     let json = JsonApiData::new(Some(id.clone()), attrs);
///
///     let expected = MyResource {
///         id: "magic_id".to_string(),
///         foo: true,
///         bar: "updated".to_string()
///     };
///
///     let result = MyResource::update(id.clone(), json.clone(), &Default::default(), MyCtx {});
///     assert_eq!(expected.into_json(&Default::default()), result.unwrap());
///
///
///     let id = "some_other_id".to_string();
///     let result = MyResource::update(id, json, &Default::default(), MyCtx {});
///     let err = Err((MyError("Cannot patch resource!".to_string()), Status::BadRequest));
///     assert_eq!(err, result);
/// }
/// ```
pub trait JsonPatch: Handler
where
    Self: JsonApiResource + ToJson
{
    /// A user-defined error type
    type Error: std::error::Error + Send;

    /// A user-defined type
    type Context;

    /// Attempts to update a resource with the given JSON patch.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate rustiful;
    /// #
    /// # #[macro_use]
    /// # extern crate serde_derive;
    /// #
    /// # #[macro_use]
    /// # extern crate rustiful_derive;
    /// #
    /// # use std::error::Error;
    /// # use std::fmt::Display;
    /// # use rustiful::JsonPatch;
    /// # use rustiful::ToJson;
    /// # use rustiful::IntoJson;
    /// # use rustiful::JsonApiData;
    /// # use rustiful::JsonApiParams;
    /// # use rustiful::iron::status::Status;
    ///
    /// #[derive(Debug, PartialEq, Eq, JsonApi, Default, Clone)]
    /// struct MyResource {
    ///     id: String,
    ///     foo: bool,
    ///     bar: String
    /// }
    ///
    /// # #[derive(Debug, PartialEq, Eq)]
    /// # struct MyError(String);
    /// #
    /// # impl Error for MyError {
    /// #    fn description(&self) -> &str {
    /// #        &self.0
    /// #    }
    /// # }
    /// #
    /// # impl Display for MyError {
    /// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    /// #        self.0.fmt(f)
    /// #    }
    /// # }
    /// #
    /// # struct MyCtx {
    /// # }
    /// #
    /// impl JsonPatch for MyResource {
    /// #    type Error = MyError;
    /// #    type Context = MyCtx;
    /// #
    /// #    fn update(id: Self::JsonApiIdType,
    /// #              json: JsonApiData<Self>,
    /// #              params: &Self::Params,
    /// #              ctx: Self::Context)
    /// #              -> Result<JsonApiData<Self>, (Self::Error, Status)> {
    /// #        Err((MyError("This is just here to demonstrate patch".to_string()), Status::BadRequest))
    /// #    }
    /// }
    ///
    /// fn main() {
    ///     let resource = MyResource {
    ///         id: "magic_id".to_string(),
    ///         foo: true,
    ///         bar: "not_updated_yet".to_string()
    ///     };
    ///
    ///     let id = "magic_id".to_string();
    ///     let attrs = <MyResource as ToJson>::Attrs::new(None, None);
    ///     let mut json = JsonApiData::new(Some(id.clone()), attrs);
    ///
    ///     // Nothing gets changed here
    ///     let update_with_nones = resource.clone().patch(json.clone());
    ///     assert_eq!(resource.clone(), update_with_nones.unwrap());
    ///
    ///     // Let's do an update here
    ///     let updated = MyResource {
    ///         id: "magic_id".to_string(),
    ///         foo: false,
    ///         bar: "updated".to_string()
    ///     };
    ///     let new_attrs = <MyResource as ToJson>::Attrs::new(Some(false),
    ///                                                        Some("updated".to_string()));
    ///     json.attributes = new_attrs;
    ///
    ///     let update_with_somes = resource.clone().patch(json);
    ///     assert_eq!(updated, update_with_somes.unwrap());
    /// }
    ///
    /// ```
    fn patch(self, json: JsonApiData<Self>) -> Result<Self, String>
    where
        Self: TryFrom<(Self, JsonApiData<Self>), Error = String>
    {
        (self, json).try_into()
    }

    /// Updates a resource.
    ///
    /// * `id` - The id of the resource to delete. `Self::JsonApiIdType` is the
    /// type of the field of `Self` which is either named `id` or which has the `JsonApiId`
    /// attribute. In other words, if `Self` has a field named `id` which is a `Uuid`, then
    /// `Self::JsonApiIdType` will be `Uuid`. The only requirement is that `Self::JsonApiIdType` is
    /// convertible to a String (i.e the type implements `FromStr`).
    /// * `json` - The JSON patch with the attributes to update.
    /// * `params` - A type-safe reference of the params
    /// passed in the request. `Self::FilterField` and `Self::SortField` are automatically
    /// implemented when `JsonApi` is derived. See `JsonApiParams` for more info.
    /// * `ctx` - A user defined context type. This is used to instantiate the given
    /// type on each request. This type can be used for whatever you like, such as an auth token
    /// or a database connection.
    fn update(
        id: Self::JsonApiIdType,
        json: JsonApiData<Self>,
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<JsonApiData<Self>, (Self::Error, Self::Status)>;
}

/// A trait for implementing GET `/{resource-name}` on a resource type.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate rustiful_derive;
///
/// use std::error::Error;
/// use std::fmt::Display;
/// use rustiful::JsonIndex;
/// use rustiful::ToJson;
/// use rustiful::IntoJson;
/// use rustiful::JsonApiData;
/// use rustiful::JsonApiParams;
///
/// #[derive(Debug, PartialEq, Eq, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # #[derive(Debug, PartialEq, Eq)]
/// # struct MyError(String);
/// #
/// # impl Error for MyError {
/// #    fn description(&self) -> &str {
/// #        &self.0
/// #    }
/// # }
/// #
/// # impl Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
/// #        self.0.fmt(f)
/// #    }
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// impl JsonIndex for MyResource {
///     type Error = MyError;
///     type Context = MyCtx;
///
///     fn find_all(params: &Self::Params,
///                 ctx: Self::Context)
///                 -> Result<Vec<JsonApiData<Self>>, (Self::Error, Self::Status)> {
///         let resource = MyResource {
///             id: "magic_id".to_string(),
///             foo: true,
///             bar: "hello".to_string()
///         };
///
///         Ok(vec![resource].into_json(params))
///     }
/// }
///
/// fn main() {
///     let id = "magic_id".to_string();
///     let attrs = <<MyResource as ToJson>::Attrs>::new(Some(true), Some("hello".to_string()));
///     let resource = MyResource::find_all(&Default::default(), MyCtx {});
///     let expected = JsonApiData::new(Some(id), attrs);
///     assert_eq!(vec![expected], resource.unwrap());
/// }
/// ```
pub trait JsonIndex: Handler
where
    Self: JsonApiResource + ToJson
{
    /// A user-defined error type
    type Error: std::error::Error + Send;

    /// A user-defined type
    type Context;

    /// Gets a collection of resources.
    ///
    /// * `params` - A type-safe reference of the params
    /// passed in the request. `Self::FilterField` and `Self::SortField` are automatically
    /// implemented when `JsonApi` is derived. See `JsonApiParams`
    /// * `ctx` - A user defined context type. This is used to instantiate the given
    /// type on each request. This type can be used for whatever you like, such as an auth token
    /// or a database connection.
    fn find_all(
        params: &Self::Params,
        ctx: Self::Context
    ) -> Result<Vec<JsonApiData<Self>>, (Self::Error, Self::Status)>;
}

/// A trait for implementing DELETE `/{resource-name}/{id}` on a resource type.
///
/// # Example
///
/// ```
/// extern crate rustiful;
///
/// #[macro_use]
/// extern crate serde_derive;
///
/// #[macro_use]
/// extern crate rustiful_derive;
///
/// use std::error::Error;
/// use std::fmt::Display;
/// use rustiful::JsonDelete;
/// use rustiful::ToJson;
/// use rustiful::IntoJson;
/// use rustiful::JsonApiData;
/// use rustiful::JsonApiParams;
/// use rustiful::iron::status::Status;
///
/// #[derive(Debug, PartialEq, Eq, JsonApi, Default)]
/// struct MyResource {
///     id: String,
///     foo: bool,
///     bar: String
/// }
///
/// # #[derive(Debug, PartialEq, Eq)]
/// # struct MyError(String);
/// #
/// # impl Error for MyError {
/// #    fn description(&self) -> &str {
/// #        &self.0
/// #    }
/// # }
/// #
/// # impl Display for MyError {
/// #    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
/// #        self.0.fmt(f)
/// #    }
/// # }
/// #
/// # struct MyCtx {
/// # }
/// #
/// impl JsonDelete for MyResource {
///     type Error = MyError;
///     type Context = MyCtx;
///
///     fn delete(id: Self::JsonApiIdType, ctx: Self::Context) -> Result<(), (Self::Error, Status)> {
///         let resource = MyResource {
///             id: "magic_id".to_string(),
///             foo: true,
///             bar: "hello".to_string()
///         };
///
///         if id == resource.id {
///             Ok(())
///         } else {
///             Err((MyError("Invalid id!".to_string()), Status::BadRequest))
///         }
///     }
/// }
///
/// fn main() {
///     let id = "magic_id".to_string();
///     let err = Err((MyError("Invalid id!".to_string()), Status::BadRequest));
///     assert_eq!(Ok(()), MyResource::delete("magic_id".to_string(), MyCtx {}));
///     assert_eq!(err, MyResource::delete("other_id".to_string(), MyCtx {}));
/// }
/// ```
pub trait JsonDelete: Handler
where
    Self: JsonApiResource
{
    /// A user-defined error type
    type Error: std::error::Error + Send;

    /// A user-defined type
    type Context;

    /// Deletes a resource.
    ///
    /// * `id` - The id of the resource to delete. `Self::JsonApiIdType` is the
    /// type of the field of `Self` which is either named `id` or which has the `JsonApiId`
    /// attribute. In other words, if `Self` has a field named `id` which is a `Uuid`, then
    /// `Self::JsonApiIdType` will be `Uuid`. The only requirement is that `Self::JsonApiIdType` is
    /// convertible to a String (i.e the type implements `FromStr`).
    /// * `ctx` - A user defined context type. This is used to instantiate the given
    /// type on each request. This type can be used for whatever you like, such as an auth token
    /// or a database connection.
    fn delete(
        id: Self::JsonApiIdType,
        ctx: Self::Context
    ) -> Result<(), (Self::Error, Self::Status)>;
}
