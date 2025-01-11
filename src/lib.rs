pub use error::Error;
pub use frame::Frame;
pub use series::{Series, SeriesData};

pub type Result<T> = std::result::Result<T, Error>;

mod error;
mod frame;
mod series;
