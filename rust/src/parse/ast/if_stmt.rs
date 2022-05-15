use crate::parse::ast::Statement::IF_STATEMENT;
use crate::parse::ast::{AstNode, CodeBlock, Expression};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IfStmt {
    pub expr: Expression,
    pub then: CodeBlock,
    pub else_stmt: Option<CodeBlock>,
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
            Some(else_pair) => {
                let else_inner = else_pair
                    .into_inner()
                    .next()
                    .ok_or(ParseError::ExpectedInner)?;
                match else_inner.clone().as_rule() {
                    Rule::code_block => Some(CodeBlock::try_from(else_inner)?),
                    Rule::if_stmt => Some(CodeBlock {
                        statements: vec![IF_STATEMENT(IfStmt::try_from(else_inner)?)],
                    }),
                    _ => return Err(ParseError::NoMatch(String::from("IfStmt::from_pair"))),
                }
            }
        };

        return Ok(Self {
            expr,
            then,
            else_stmt,
        });
    }
}
