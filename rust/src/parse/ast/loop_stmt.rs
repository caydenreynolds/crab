use std::convert::TryFrom;
use pest::iterators::Pair;
use crate::parse::ast::{AstNode, CodeBlock, Expression};
use crate::parse::{Rule, Result, ParseError};
use crate::try_from_pair;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub expr: Expression,
    pub then: CodeBlock,
}

try_from_pair!(DoWhileStmt, Rule::do_while_stmt);
impl AstNode for DoWhileStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        return Ok(Self { expr, then });
    }
}
