use crate::parse::ast::{AstNode, CrabType, Expression, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct NamedFnParam {
    pub name: Ident,
    pub crab_type: CrabType,
    pub expr: Expression,
}

try_from_pair!(NamedFnParam, Rule::named_fn_param);
impl AstNode for NamedFnParam {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self {
            name,
            crab_type,
            expr,
        })
    }
}
