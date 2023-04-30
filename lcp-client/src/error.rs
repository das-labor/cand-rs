use lcp_proto::ErrorCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Protocol Error: {0}")]
    ProtocolError(#[from] lcp_proto::Error),

    #[error("Invalid Response ID")]
    InvalidResponseId,

    #[error("Unexpected Response Type")]
    UnexpectedResponseType,

    #[error("Timeout while waiting for response")]
    ResponeTimeout,

    #[error("Server Error: {message}")]
    ServerError { code: ErrorCode, message: String },
}
