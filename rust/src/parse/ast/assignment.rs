use std::convert::TryFrom;
use pest::iterators::Pair;
use crate::parse::ast::{AstNode, Expression, Ident};
use crate::parse::{Rule, ParseError, Result};

#[derive(Debug, Clone)]
pub struct Assignment {
    pub var_name: Ident,
    pub expr: Expression,
}

/// Assignment requries a custom TryFrom implementation because it can be built from two different rules
impl TryFrom<Pair<'_, Rule>> for Assignment {
    type Error = ParseError;
    fn try_from(pair: Pair<Rule>) -> std::result::Result<Assignment, Self::Error> {
        match pair.as_rule() {
            Rule::assignment => Assignment::from_pair(pair),
            Rule::reassignment => Assignment::from_pair(pair),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(Assignment)),
                format!(
                    "{} or {}",
                    stringify!(Rule::assignment),
                    stringify!(Rule::reassignment)
                ),
                format!("{:?}", pair.as_rule()),
            )),
        }
    }
}
impl AstNode for Assignment {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let var_name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { var_name, expr })
    }
}
