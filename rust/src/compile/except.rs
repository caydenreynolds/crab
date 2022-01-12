use crate::parse::CrabType;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, CompileError>;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Invalid return type for function. Expected {0:?}, instead got {1:?}")]
    InvalidReturn(CrabType, CrabType),

    #[error("Got a none option when one was expected")]
    InvalidNoneOption,

    #[error("Could not find function with name {0}")]
    CouldNotFindFunction(String),

    #[error(
        "Cannot assign variable with name {0}, because a variable with that name already exists"
    )]
    VarAlreadyExists(String),

    #[error("Variable with name {0} does not exist")]
    NoVar(String),
}
