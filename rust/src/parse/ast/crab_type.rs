use crate::compile::CompileError;
use crate::parse::ast::{AstNode, Ident, StructIdent};
use crate::parse::{ParseError, Rule};
use crate::{parse, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    STRUCT(StructIdent), // Struct encompasses both structs and interfaces
    LIST(Box<CrabType>),
    VOID,
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let next = pair.into_inner().next().ok_or(ParseError::ExpectedInner)?;
        match next.clone().as_rule() {
            Rule::ident => Ok(Self::STRUCT(StructIdent::try_from(next)?)),
            Rule::crab_type => Ok(Self::LIST(Box::new(CrabType::try_from(next)?))),
            _ => Err(ParseError::NoMatch(String::from("CrabType::from_pair"))),
        }
    }
}

impl Display for CrabType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrabType::STRUCT(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
            CrabType::VOID => write!(f, "VOID"),
        }?;

        Ok(())
    }
}
