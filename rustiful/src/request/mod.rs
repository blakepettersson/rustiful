pub mod get;
pub mod post;
pub mod index;
pub mod patch;
pub mod delete;

pub use self::get::*;
pub use self::index::*;
pub use self::patch::*;
pub use self::post::*;
pub use self::delete::*;

use super::status::Status;
