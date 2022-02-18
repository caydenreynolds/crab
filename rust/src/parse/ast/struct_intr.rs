use crate::parse::ast::{AstNode, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructIntr {
    pub struct_name: Ident,
    pub inters: Vec<Ident>,
}

try_from_pair!(StructIntr, Rule::intr_block);
impl AstNode for StructIntr {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("StructIntr::from_pair")))?
                .as_str(),
        );
        let mut inters = vec![];

        for in_pair in inner {
            inters.push(Ident::from(in_pair.as_str()));
        }

        Ok(Self {
            struct_name,
            inters,
        })
    }
}
