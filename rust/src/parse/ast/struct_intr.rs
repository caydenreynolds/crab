use crate::parse::ast::{AstNode, Ident, StructIdent};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructIntr {
    pub struct_id: StructIdent,
    pub inters: Vec<StructIdent>,
}

try_from_pair!(StructIntr, Rule::intr_block);
impl AstNode for StructIntr {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_id = StructIdent::try_from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?)?;
        let inters = inner.map(|pair| StructIdent::try_from(pair)).collect::<Result<Vec<StructIdent>>>()?;

        Ok(Self {
            struct_id,
            inters,
        })
    }
}
