use crate::parse::ast::{AstNode, FuncSignature, Ident, StructIdent};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabInterface {
    pub id: StructIdent,
    pub fns: Vec<FuncSignature>,
}

try_from_pair!(CrabInterface, Rule::interface);
impl AstNode for CrabInterface {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let id = StructIdent::try_from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?)?;
        let mut fns = vec![];

        for in_pair in inner {
            fns.push(FuncSignature::try_from(in_pair)?);
        }

        Ok(Self { id, fns })
    }
}
