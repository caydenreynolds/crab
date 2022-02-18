use crate::parse::ast::{AstNode, Expression, Ident, NamedExpression};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FnCall {
    pub name: Ident,
    pub unnamed_args: Vec<Expression>,
    pub named_args: Vec<NamedExpression>,
}

try_from_pair!(FnCall, Rule::fn_call);
impl AstNode for FnCall {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let mut named_args = vec![];
        let mut unnamed_args = vec![];
        let mut seen_named_arg = false;

        for inner_pair in inner {
            match inner_pair.clone().as_rule() {
                Rule::expression => {
                    if seen_named_arg {
                        return Err(ParseError::PositionalArgAfterNamedParam(name.clone()));
                    }
                    unnamed_args.push(Expression::try_from(inner_pair)?);
                }
                Rule::named_expression => {
                    named_args.push(NamedExpression::try_from(inner_pair)?);
                    seen_named_arg = true;
                }
                _ => return Err(ParseError::NoMatch(String::from("FnCall::from_pair"))),
            }
        }

        Ok(Self {
            name,
            named_args,
            unnamed_args,
        })
    }
}
