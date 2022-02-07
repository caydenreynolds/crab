use std::convert::TryFrom;
use pest::iterators::Pair;
use crate::parse::ast::{AstNode, CodeBlock, Expression};
use crate::parse::{Rule, Result, ParseError};
use crate::try_from_pair;

#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
pub enum ElseStmt {
    ELSE(CodeBlock),
    ELIF(IfStmt),
}

try_from_pair!(ElseStmt, Rule::else_stmt);
impl AstNode for ElseStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let next = inner.next().ok_or(ParseError::ExpectedInner)?;
        return match next.clone().as_rule() {
            Rule::code_block => Ok(ElseStmt::ELSE(CodeBlock::try_from(next)?)),
            Rule::if_stmt => Ok(ElseStmt::ELIF(IfStmt::try_from(next)?)),
            _ => Err(ParseError::NoMatch(String::from("ElseStmt::from_pair"))),
        };
    }
}
