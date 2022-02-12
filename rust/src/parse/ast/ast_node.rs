use crate::parse::{Result, Rule};
use pest::iterators::Pair;

pub trait AstNode {
    // Build an instance of self from the given pair, assuming that the pair's rule type is correct
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized;
}

/*
 * Generates a try-from function for getting the given crabNode from the specified rule type
 * The crabNode must implement NodeFromPair for the given rule type
 */
#[macro_export]
macro_rules! try_from_pair {
    ($crabNode:ty, $rule:path) => {
        impl TryFrom<Pair<'_, Rule>> for $crabNode {
            type Error = ParseError;
            fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
                match pair.as_rule() {
                    $rule => <$crabNode>::from_pair(pair),
                    _ => Err(ParseError::IncorrectRule(
                        String::from(stringify!($crabNode)),
                        String::from(stringify!($rule)),
                        format!("{:?}", pair.as_rule()),
                    )),
                }
            }
        }
    };
}
