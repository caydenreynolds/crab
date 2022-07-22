use crate::parse::ast::{AstNode, CodeBlock, Expression, Ident};
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Assignment {
    pub var_name: Ident,
    pub expr: Expression,
}

/// Assignment requires a custom TryFrom implementation because it can be built from two different rules
impl TryFrom<Pair<'_, Rule>> for Assignment {
    type Error = ParseError;
    fn try_from(pair: Pair<Rule>) -> std::result::Result<Assignment, Self::Error> {
        match pair.as_rule() {
            Rule::assignment => Assignment::from_pair(pair),
            Rule::reassignment => Assignment::from_pair(pair),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(Assignment)),
                format!(
                    "{} or {}",
                    stringify!(Rule::assignment),
                    stringify!(Rule::reassignment)
                ),
                format!("{:?}", pair.as_rule()),
            )),
        }
    }
}
impl AstNode for Assignment {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let var_name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { var_name, expr })
    }
}

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
                        statements: vec![Statement::IF_STATEMENT(IfStmt::try_from(else_inner)?)],
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
