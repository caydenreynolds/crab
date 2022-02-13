use crate::parse::ast::Ident;
use crate::parse::Rule;
use std::num::ParseIntError;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ParseError>;

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

    #[error("Modifier is not recognized as a valid access modifier. Use 'pro', 'pub', or ''. Does your grammer.pest match your parser.old?")]
    BadAccessModifier,

    #[error("Program cannot have any AST nodes placed to the right of it")]
    ProgramRight,

    #[error("This AST node cannot be set because it is already set")]
    NodeAlreadySet,

    #[error("{0} is not a valid CrabType")]
    InvalidCrabType(String),

    #[error("Function {0} has positional param {1} after a named param")]
    PositionalParamAfterNamedParam(Ident, Ident),

    #[error("Call of function {0} has positional param after a named param")]
    PositionalArgAfterNamedParam(Ident),

    #[error("The main function does not have the expected signature. Should be: fn main() -> Int")]
    MainSignature,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),

    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
}
