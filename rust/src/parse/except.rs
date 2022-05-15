use crate::parse::ast::Ident;
use crate::parse::Rule;
use std::num::ParseIntError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("This is a dummy error for when no error occurred but a function returned no result. Useful to avoid wrapping an option in a result")]
    None,

    #[error("Could not build a crab tree because the parser did not find a match in function {0}")]
    NoMatch(String),

    #[error("Failed to convert pair to {0} because the rule type is incorrect. Expected {1}, instead got {2}")]
    IncorrectRule(String, String, String),

    #[error("Failed to iterate over a match's inner because a value was expected but none found")]
    ExpectedInner,

    #[error(
        "Failed to process a match's inner because a value was not expected, but one was found"
    )]
    UnexpectedInner,

    #[error("Function {0} has positional param {1} after a named param")]
    PositionalParamAfterNamedParam(Ident, Ident),

    #[error("Call of function {0} has positional param after a named param")]
    PositionalArgAfterNamedParam(Ident),

    #[error("The main function does not have the expected signature. Should be: fn main() -> Int")]
    MainSignature,

    #[error("The struct {0} does not implement {1}, which is required by interface {2}")]
    DoesNotImplement(Ident, Ident, Ident),

    #[error("The interface {0} does not exist")]
    InterfaceNotFound(Ident),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
}
