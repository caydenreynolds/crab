use crate::parse::ast::{AstNode, CodeBlock, ElseStmt, Expression};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IfStmt {
    pub expr: Expression,
    pub then: CodeBlock,
    pub else_stmt: Option<Box<ElseStmt>>,
}

try_from_pair!(IfStmt, Rule::if_stmt);
impl AstNode for IfStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let else_stmt = match inner.next() {
            None => None,
            Some(else_pair) => Some(Box::new(ElseStmt::try_from(else_pair)?)),
        };

        return Ok(Self {
            expr,
            then,
            else_stmt,
        });
    }
}
