mod identity;
mod result;
mod sidecar;

pub use identity::{Identity, IdentityError};
pub use result::{DiffBounds, Entry, ResultFile, Status, Summary};
pub use sidecar::{EnvInfo, ImageInfo, Sidecar, SourceInfo};

pub const SCHEMA_VERSION: u32 = 1;
