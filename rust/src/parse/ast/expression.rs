use std::convert::TryFrom;
use pest::iterators::Pair;
use crate::parse::ast::{AstNode, FnCall, Ident, Primitive, StructInit};
use crate::parse::{Rule, Result, ParseError};
use crate::try_from_pair;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Expression {
    PRIM(Primitive),
    FN_CALL(FnCall),
    VARIABLE(Ident),
    STRUCT_INIT(StructInit),
}

try_from_pair!(Expression, Rule::expression);
#[allow(unreachable_patterns)]
impl AstNode for Expression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::primitive => Ok(Expression::PRIM(Primitive::try_from(expr_type)?)),
            Rule::fn_call => Ok(Expression::FN_CALL(FnCall::try_from(expr_type)?)),
            Rule::ident => Ok(Expression::VARIABLE(Ident::from(expr_type.as_str()))),
            Rule::struct_init => Ok(Expression::STRUCT_INIT(StructInit::try_from(expr_type)?)),
            _ => Err(ParseError::NoMatch(String::from("Expression::from_pair"))),
        };
    }
}
