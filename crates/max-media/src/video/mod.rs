//! Everything here must be used from the main thread, as the system's video framework requires.

mod clip;
mod frame;
mod protected;
mod sound;
mod time;

pub use clip::Clip;
pub use frame::Frame;
pub use protected::{FetchKey, ProtectedTitle, ProtectedVideo};
