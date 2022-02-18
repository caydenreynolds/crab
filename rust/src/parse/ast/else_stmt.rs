use crate::parse::ast::{AstNode, CodeBlock, IfStmt};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
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
