use crate::parse::ast::{AstNode, Ident, StructFieldInit};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct StructInit {
    pub name: Ident,
    pub fields: Vec<StructFieldInit>,
}

try_from_pair!(StructInit, Rule::struct_init);
impl AstNode for StructInit {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
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
            fields.push(StructFieldInit::try_from(in_pair)?);
        }

        Ok(Self { name, fields })
    }
}
