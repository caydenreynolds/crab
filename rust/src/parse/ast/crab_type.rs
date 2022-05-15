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
    UINT8,
    UINT64,
    STRING,
    VOID,
    FLOAT,
    BOOL,
    STRUCT(Ident), // TODO: Struct currently encompasses both structs and interfaces
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
            Rule::type_name => match next.as_str() {
                "__uint64__" => Ok(Self::UINT64),
                "__uint8__" => Ok(Self::UINT8),
                "__string__" => Ok(Self::STRING),
                "Float" => Ok(Self::FLOAT),
                "__bool__" => Ok(Self::BOOL),
                s => Ok(Self::STRUCT(Ident::from(s))),
            },
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
            CrabType::UINT8 => write!(f, "UINT8"),
            CrabType::UINT64 => write!(f, "UINT64"),
            CrabType::STRING => write!(f, "STRING"),
            CrabType::VOID => write!(f, "VOID"),
            CrabType::FLOAT => write!(f, "FLOAT"),
            CrabType::BOOL => write!(f, "BOOL"),
            CrabType::STRUCT(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
        }?;

        Ok(())
    }
}
