use crate::parse::ast::{AstNode, FuncSignature, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabInterface {
    pub name: Ident,
    pub fns: Vec<FuncSignature>,
}

try_from_pair!(CrabInterface, Rule::interface);
impl AstNode for CrabInterface {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from(
                    "CrabInterface::from_pair",
                )))?
                .as_str(),
        );
        let mut fns = vec![];

        for in_pair in inner {
            fns.push(FuncSignature::try_from(in_pair)?);
        }

        Ok(Self { name, fns })
    }
}
