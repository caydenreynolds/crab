use crate::parse::ast::{CrabType, Ident};
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

    #[error("Function {0} requires {1} positional arguments, but {2} were supplied")]
    PositionalArgumentCount(Ident, usize, usize),

    #[error("Function {0} does not have a named argument with name {1}")]
    InvalidNamedArgument(Ident, Ident),

    #[error(
        "Cannot assign variable with name {0}, because a variable with that name already exists"
    )]
    VarAlreadyExists(Ident),

    #[error("Variable with name {0} does not exist")]
    VarDoesNotExist(Ident),

    #[error("Cannot assign a {2:?} to variable {0} because the variable type is {1:?}")]
    VarType(Ident, CrabType, CrabType),

    #[error("The following error occurred while building a malloc operation: {0}")]
    MallocErr(String),

    #[error("A function may not accept an argument of type {0}")]
    InvalidArgType(String),

    #[error("{0}")]
    Internal(String),

    #[error("Failed to get var value type as {0}")]
    VarValueType(String),

    #[error("Struct {0} has multiple definitions")]
    StructRedefinition(Ident),

    #[error("Function {0} is declared multiple times")]
    FunctionRedefinition(Ident),

    #[error("Struct with name {0} does not exist")]
    StructDoesNotExist(Ident),

    #[error("Failed to pop a value off of stack {0} because it is empty")]
    EmptyStack(String),

    #[error("Failed to build function {0} because it does not always return a value")]
    NoReturn(Ident),

    #[error("Initialization of struct {0} expected {1} fields, instead got {2}")]
    StructInitFieldCount(Ident, usize, usize),

    #[error("Initialization of struct {0} expects field {1}, which has not been supplied")]
    StructInitFieldName(Ident, Ident),

    #[error("Struct {0} does not contain a field with name {1}")]
    StructFieldName(Ident, Ident),
}
