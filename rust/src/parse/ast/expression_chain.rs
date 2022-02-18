use crate::parse::ast::{AstNode, ExpressionChainType};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExpressionChain {
    pub this: ExpressionChainType,
    pub next: Option<Box<ExpressionChain>>,
}

try_from_pair!(ExpressionChain, Rule::expression_chain);
impl AstNode for ExpressionChain {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let this = ExpressionChainType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let mut expr_chain = Self {
            this: this,
            next: None,
        };

        for in_pair in inner {
            let next = ExpressionChainType::try_from(in_pair)?;
            expr_chain = expr_chain.append(next);
        }

        Ok(expr_chain)
    }
}
impl ExpressionChain {
    fn append(self, addition: ExpressionChainType) -> Self {
        match self.next {
            None => Self {
                this: self.this,
                next: Some(Box::new(Self {
                    this: addition,
                    next: None,
                })),
            },
            Some(ec) => Self {
                this: self.this,
                next: Some(Box::new(ec.append(addition))),
            },
        }
    }
}
