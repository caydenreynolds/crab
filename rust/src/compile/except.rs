use crate::parse::ast::{CrabType, Ident, StructId};
use crate::quill::QuillError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Struct {0} expected to have a caller id, but it did not")]
    NoCallerId(Ident),

    #[error("A template got the wrong number of type arguments. Expected{0} and got {1}")]
    WrongTemplateTypeCount(usize, usize),

    #[error("Got a void type where one was not expected")]
    VoidType,

    #[error("Could not find function with name {0}")]
    CouldNotFindFunction(Ident),

    #[error("Function {0} requires {1} positional arguments, but {2} were supplied")]
    PositionalArgumentCount(Ident, usize, usize),

    #[error("Function {0} argument {1} expects type {2}, instead got {3}")]
    ArgumentType(Ident, Ident, CrabType, CrabType),

    #[error("Could not get value with name {0} as a struct type in {1}")]
    NotAStruct(StructId, String),

    #[error("Could not get a value as an interface type")]
    NotAnInterface,

    #[error(
        "Cannot assign variable with name {0}, because a variable with that name already exists"
    )]
    VarAlreadyExists(Ident),

    #[error("Variable with name {0} does not exist")]
    VarDoesNotExist(Ident),

    #[error("Struct {0} has multiple definitions")]
    StructRedefinition(Ident),

    #[error("Interface {0} is declared multiple times")]
    InterfaceRedefinition(Ident),

    #[error("Struct with name {0} does not exist")]
    StructDoesNotExist(StructId),

    #[error("Type with name {0} does not exist")]
    TypeDoesNotExist(Ident),

    #[error("Failed to build function {0} because it does not always return a value")]
    NoReturn(Ident),

    #[error("Initialization of struct {0} expects field {1}, which has not been supplied")]
    StructInitFieldName(Ident, Ident),

    #[error("Struct {0} does not contain a field with name {1}")]
    StructFieldName(CrabType, Ident),

    #[error("No main function found")]
    NoMain,

    #[error("Function expected argument with name {0}, but none was supplied")]
    ArgumentNotSupplied(Ident),

    #[error(transparent)]
    QuillErr(#[from] QuillError),
}
