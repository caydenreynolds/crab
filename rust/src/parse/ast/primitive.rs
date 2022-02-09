use pest::iterators::Pair;
use crate::parse::{Rule, parse_string, Result, ParseError};
use crate::parse::ast::AstNode;
use crate::try_from_pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Primitive {
    UINT(u64),
    STRING(String),
    BOOL(bool),
}

try_from_pair!(Primitive, Rule::primitive);
#[allow(unreachable_patterns)]
impl AstNode for Primitive {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let prim_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match prim_type.as_rule() {
            Rule::uint64_primitive => Ok(Primitive::UINT(prim_type.as_str().parse()?)),
            Rule::string_primitive => Ok(Primitive::STRING(parse_string(
                prim_type
                    .into_inner()
                    .next()
                    .ok_or(ParseError::ExpectedInner)?
                    .as_str(),
            ))),
            Rule::bool_primitive => Ok(Primitive::BOOL(prim_type.as_str() == "true")),
            _ => Err(ParseError::NoMatch(String::from("Primitive::from_pair"))),
        };
    }
}
