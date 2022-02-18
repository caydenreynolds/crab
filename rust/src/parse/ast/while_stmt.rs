use crate::parse::ast::{AstNode, CodeBlock, Expression};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WhileStmt {
    pub expr: Expression,
    pub then: CodeBlock,
}

try_from_pair!(WhileStmt, Rule::while_stmt);
impl AstNode for WhileStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        return Ok(Self { expr, then });
    }
}
