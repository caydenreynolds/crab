use crate::parse::Rule;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("This is a dummy error for when no error occurred but a function returned no result. Useful to avoid wrapping an option in a result")]
    None,

    #[error("Could not build a crab tree because the parser did not find a match")]
    NoMatch,

    #[error("Failed to convert pair to {0} because the rule type is incorrect. Expected {1}, instead got {2}")]
    IncorrectRule(String, String, String),

    #[error("Failed to iterate over a match's inner because a value was expected but none found")]
    ExpectedInner,

    #[error("Modifier is not recognized as a valid access modifier. Use 'pro', 'pub', or ''. Does your grammer.pest match your parser.old?")]
    BadAccessModifier,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),

    #[error("Program cannot have any AST nodes placed to the right of it")]
    ProgramRight,

    #[error("This AST node cannot be set because it is already set")]
    NodeAlreadySet,
}
