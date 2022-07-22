use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use pest::iterators::Pair;
use crate::compile::CompileError;
use crate::{compile, parse, try_from_pair};
use crate::parse::ast::AstNode;
use crate::parse::{ParseError, Rule};
use crate::util::ListFunctional;

pub type Ident = String;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    VOID,
    SIMPLE(Ident),
    LIST(Box<CrabType>),
    TMPL(Ident, Vec<CrabType>),
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let next = pairs.next().ok_or(ParseError::ExpectedInner)?;
        match next.as_rule() {
            Rule::simple_crab_type => Ok(Self::SIMPLE(SimpleCrabType::try_from(next)?.0)),
            Rule::list_crab_type => Ok(Self::LIST(ListCrabType::try_from(next)?.0)),
            Rule::tmpl_crab_type => {
                let tct = TmplCrabType::try_from(next)?;
                Ok(Self::TMPL(tct.0, tct.1))
            },
            _ => Err(ParseError::NoMatch(String::from("CrabType::from_pair"))),
        }
    }
}
impl CrabType {
    pub fn try_get_struct_name(&self) -> compile::Result<Ident> {
        match self {
            Self::SIMPLE(id) => Ok(id.clone()),
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
            CrabType::SIMPLE(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
            CrabType::TMPL(i, t) => {
                write!(f, "TMPL_{}", i)?;
                for ct in t {
                    write!(f, "_{}", ct)?;
                }
                Ok(())
            }
        }
    }
}

struct SimpleCrabType(Ident);
try_from_pair!(SimpleCrabType, Rule::simple_crab_type);
impl AstNode for SimpleCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        Ok(Self(Ident::from(pairs.next().ok_or(ParseError::ExpectedInner)?.as_str())))
    }
}

struct ListCrabType(Box<CrabType>);
try_from_pair!(ListCrabType, Rule::list_crab_type);
impl AstNode for ListCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        Ok(Self(Box::new(CrabType::try_from(pairs.next().ok_or(ParseError::ExpectedInner)?)?)))
    }
}

struct TmplCrabType(Ident, Vec<CrabType>);
try_from_pair!(TmplCrabType, Rule::tmpl_crab_type);
impl AstNode for TmplCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        let mut inner = pair.into_inner();
        let name = Ident::from(pairs.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let tmpls = inner.try_fold(vec![], |tmpls, tmpl| {
            parse::Result::Ok(tmpls.fpush(CrabType::try_from(tmpl)?))
        })?;
        Ok(Self(name, tmpls))
    }
}
