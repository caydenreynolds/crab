use crate::parse::ast::{AstNode, Expression};
use crate::parse::{parse_string, ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::ListFunctional;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Primitive {
    UINT(u64),
    STRING(String),
    BOOL(bool),
    LIST(Vec<Expression>),
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
            Rule::list_primitive => Ok(Primitive::LIST(ListPrimitive::try_from(prim_type)?.0)),
            _ => Err(ParseError::NoMatch(String::from("Primitive::from_pair"))),
        };
    }
}

struct ListPrimitive(Vec<Expression>);
try_from_pair!(ListPrimitive, Rule::list_primitive);
impl AstNode for ListPrimitive {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |exprs, pair| {
                Result::Ok(exprs.fpush(Expression::try_from(pair)?))
            })?,
        ))
    }
}
