use crate::parse;
use crate::parse::ast::{AstNode, Ident, StructField};
use crate::parse::{ParseError, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<StructField>,
}

try_from_pair!(Struct, Rule::crab_struct);
impl AstNode for Struct {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );
        let mut fields = vec![];

        for in_pair in inner {
            fields.push(StructField::try_from(in_pair)?);
        }

        Ok(Self { name, fields })
    }
}
