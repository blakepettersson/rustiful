/// TryFrom trait; the trait is currently experimental in Rust, so use a custom implementation for
/// now.
pub trait TryFrom<T> : Sized {
    type Err;
    fn try_from(T) -> Result<Self, Self::Err>;
}