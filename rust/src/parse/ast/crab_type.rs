use crate::compile::CompileError;
use crate::parse::ast::{AstNode, Ident};
use crate::parse::{ParseError, Rule};
use crate::{parse, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    VOID,
    STRUCT(Ident),
    LIST(Box<CrabType>),
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let next = pair.into_inner().next().ok_or(ParseError::ExpectedInner)?;
        match next.clone().as_rule() {
            Rule::ident => Ok(Self::STRUCT(Ident::from(next.as_str()))),
            Rule::crab_type => Ok(Self::LIST(Box::new(CrabType::try_from(next)?))),
            _ => Err(ParseError::NoMatch(String::from("CrabType::from_pair"))),
        }
    }
}

impl CrabType {
    pub fn try_get_struct_name(&self) -> crate::compile::Result<Ident> {
        match self {
            Self::STRUCT(id) => Ok(id.clone()),
            _ => Err(CompileError::NotAStruct(
                String::from("unknown"),
                String::from("CrabType::try_get_struct_name"),
            )),
        }
    }
}

impl Display for CrabType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrabType::VOID => write!(f, "VOID"),
            CrabType::STRUCT(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
        }?;

        Ok(())
    }
}
