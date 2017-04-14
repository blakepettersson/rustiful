pub mod get;
pub mod post;
pub mod index;
pub mod patch;
pub mod delete;

use super::status::Status;

pub use self::get::*;
pub use self::post::*;
pub use self::index::*;
pub use self::patch::*;
pub use self::delete::*;

