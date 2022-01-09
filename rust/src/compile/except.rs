use crate::parse::CrabType;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, CompileError>;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Invalid return type for function. Expected {0:?}, instead got {1:?}")]
    InvalidReturn(CrabType, CrabType),

    #[error("Got a none option when one was expected")]
    InvalidNoneOption,
}
