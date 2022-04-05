use crate::parse::ast::{Assignment, AstNode, DoWhileStmt, Expression, IfStmt, WhileStmt};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Statement {
    RETURN(Option<Expression>),
    ASSIGNMENT(Assignment),
    REASSIGNMENT(Assignment),
    EXPRESSION(Expression),
    IF_STATEMENT(IfStmt),
    WHILE_STATEMENT(WhileStmt),
    DO_WHILE_STATEMENT(DoWhileStmt),
}

try_from_pair!(Statement, Rule::statement);
#[allow(unreachable_patterns)]
impl AstNode for Statement {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::return_stmt => {
                let mut expr_inner = expr_type.into_inner();
                return if expr_inner.clone().count() == 1 {
                    Ok(Statement::RETURN(Some(Expression::try_from(
                        expr_inner.next().unwrap(),
                    )?)))
                } else if expr_inner.count() == 0 {
                    Ok(Statement::RETURN(None))
                } else {
                    Err(ParseError::UnexpectedInner)
                };
            }
            Rule::assignment => Ok(Statement::ASSIGNMENT(Assignment::try_from(expr_type)?)),
            Rule::reassignment => Ok(Statement::REASSIGNMENT(Assignment::try_from(expr_type)?)),
            Rule::expression => Ok(Statement::EXPRESSION(Expression::try_from(expr_type)?)),
            Rule::if_stmt => Ok(Statement::IF_STATEMENT(IfStmt::try_from(expr_type)?)),
            Rule::while_stmt => Ok(Statement::WHILE_STATEMENT(WhileStmt::try_from(expr_type)?)),
            Rule::do_while_stmt => Ok(Statement::DO_WHILE_STATEMENT(DoWhileStmt::try_from(
                expr_type,
            )?)),
            _ => Err(ParseError::NoMatch(String::from(
                "StatementType::from_pair",
            ))),
        };
    }
}
