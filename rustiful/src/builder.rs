/// A trait for implementing a builder for any `Default` type.
///
/// The implementing type is generated in jsonapi-derive.
pub trait JsonApiBuilder<T> where T: Default {
    fn new(model: T) -> Self;

    fn build(self) -> Result<T, String>;
}

/// A trait for setting a `JsonApiBuilder<Self>` on any type that implements `Default`.
///
/// This is used in order to access the builder easily after generating the builder.
pub trait ToBuilder where Self : Sized + Default {
    type Builder: JsonApiBuilder<Self>;
}
