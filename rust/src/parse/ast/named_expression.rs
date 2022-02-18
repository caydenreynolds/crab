use crate::parse::ast::{AstNode, Expression, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct NamedExpression {
    pub name: Ident,
    pub expr: Expression,
}

try_from_pair!(NamedExpression, Rule::named_expression);
impl AstNode for NamedExpression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { name, expr })
    }
}
