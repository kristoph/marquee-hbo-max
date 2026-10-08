use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Service(#[from] max_api::Error),
    #[error("{0} did not answer with a WebP image")]
    NotAnImage(String),
    #[error("the image could not be read or written: {0}")]
    Image(#[from] image::ImageError),
    #[error("the video could not be opened: {0}")]
    Video(&'static str),
    #[error(transparent)]
    File(#[from] io::Error),
}
