use pest::iterators::{Pair, Pairs};
use crate::parse::{ParseError, Result, Rule};

/// Returns whatever the next pair is, or an error if there are no more pairs
/// This does not consume the iterator, but it does consume the next element of the iterator
pub fn get_next<'a>(pairs: &'a mut Pairs<Rule>) -> Result<Pair<'a, Rule>> {
    pairs.next().ok_or(ParseError::ExpectedInner)
}


/// Consumes self and returns the first element of self's iterator
/// Returns an error if self has no elements, or more than 1 element
pub fn get_only(pair: Pair<Rule>) -> Result<Pair<Rule>> {
    let mut inner = pair.into_inner();
    let next = get_next(&mut inner)?;
    if inner.count() > 0 {
        Err(ParseError::TooManyInners)
    } else {
        Ok(next)
    }
}
