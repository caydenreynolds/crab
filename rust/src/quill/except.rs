use std::path::PathBuf;
use std::str::Utf8Error;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, QuillError>;

#[derive(Error, Debug)]
pub enum QuillError {
    #[error("Quill failed to generate LLVM IR. Debugging output sent to {0}")]
    QuillFailed(PathBuf),

    #[error("Struct does not have any field with name '{0}'")]
    StructHasNoField(String),

    #[error("Cannot register external fn {0}, because it is already registered")]
    DuplicateExternalFn(String),

    #[error(
        "Attempted to perform an operation on two integers of unequal bit widths: {0} and {1}"
    )]
    IntSize(u32, u32),

    #[error("Function {0} was not registered with llvm")]
    FnNotFound(String),

    #[error("Attempted to access a value that has not been set")]
    BadValueAccess,

    #[error("Failed to convert type {0} to {1}")]
    WrongType(String, String),

    #[error("Attempted to build an instruction that requires an after in a nib that does not have an after")]
    NoAfter,

    #[error("No struct exists with name {0}")]
    NoStruct(String),

    #[error("Failed to build a GEP instruction")]
    Gep,

    #[error("An error occurred while building a malloc instruction")]
    MallocErr,

    #[error("Function does not have a param with name {0}")]
    NoSuchParam(String),

    #[error("A FunctionType cannot be converted to a BasicTypeEnum")]
    FnAsBTE,

    #[error("Cannot convert a void type to a BasicTypeEnum")]
    VoidType,

    #[error("An error occured while building a memcpy instruction")]
    Memcpy,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}
