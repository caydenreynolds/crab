use crate::parse::{CrabType, Ident};
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, CompileError>;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Invalid return type for function. Expected {0:?}, instead got {1:?}")]
    InvalidReturn(CrabType, CrabType),

    #[error("Got a none option when some was expected in function {0}. This is a bug in crab")]
    InvalidNoneOption(String),

    #[error("Could not find function with name {0}")]
    CouldNotFindFunction(Ident),

    #[error(
        "Cannot assign variable with name {0}, because a variable with that name already exists"
    )]
    VarAlreadyExists(Ident),

    #[error("Variable with name {0} does not exist")]
    VarDoesNotExist(Ident),

    #[error("Cannot assign a {2:?} to variable {0} because the variable type is {1:?}")]
    VarType(Ident, CrabType, CrabType),
}
