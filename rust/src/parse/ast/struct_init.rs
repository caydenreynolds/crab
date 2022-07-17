use crate::parse::ast::{AstNode, Ident, StructFieldInit, StructIdent};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructInit {
    pub id: StructIdent,
    pub fields: Vec<StructFieldInit>,
}

try_from_pair!(StructInit, Rule::struct_init);
impl AstNode for StructInit {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let id = StructIdent::try_from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?)?;
        let mut fields = vec![];

        for in_pair in inner {
            fields.push(StructFieldInit::try_from(in_pair)?);
        }

        Ok(Self { id, fields })
    }
}
