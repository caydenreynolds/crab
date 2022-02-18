use crate::parse::ast::{AstNode, FnCall, Ident};
use crate::parse::{ParseError, Result, Rule};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ExpressionChainType {
    FN_CALL(FnCall),
    VARIABLE(Ident),
}

/// ExpressionChainType requires a custom TryFrom implementation because it can be built from two different rules
impl TryFrom<Pair<'_, Rule>> for ExpressionChainType {
    type Error = ParseError;
    fn try_from(pair: Pair<Rule>) -> std::result::Result<ExpressionChainType, Self::Error> {
        match pair.as_rule() {
            Rule::ident => ExpressionChainType::from_pair(pair),
            Rule::fn_call => ExpressionChainType::from_pair(pair),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(ExpressionChainType)),
                format!(
                    "{} or {}",
                    stringify!(Rule::ident),
                    stringify!(Rule::fn_call)
                ),
                format!("{:?}", pair.as_rule()),
            )),
        }
    }
}
impl AstNode for ExpressionChainType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        match pair.clone().as_rule() {
            Rule::ident => Ok(Self::VARIABLE(Ident::from(pair.as_str()))),
            Rule::fn_call => Ok(Self::FN_CALL(FnCall::try_from(pair)?)),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(ExpressionChainType)),
                format!(
                    "{:#?} or {:#?}",
                    stringify!(Rule::ident),
                    stringify!(Rule::fn_call)
                ),
                format!("{:#?}", pair.as_rule()),
            )),
        }
    }
}
