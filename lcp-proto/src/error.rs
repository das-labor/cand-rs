use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Object too large for protocol")]
    ObjectTooLarge,

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid ID in protocol")]
    InvalidId,

    #[error("CBOR Serialization Error: {0}")]
    CBORSerError(#[from] ciborium::ser::Error<std::io::Error>),

    #[error("CBOR Deserialization Error: {0}")]
    CBORDeError(#[from] ciborium::de::Error<std::io::Error>),

    #[error("Invalid UTF-8")]
    UTF8Error,
}
