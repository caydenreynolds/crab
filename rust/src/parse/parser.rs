use crate::compile;
use crate::compile::AstVisitor;
use crate::parse::{ParseError, Result};
use log::trace;
use pest::{iterators::Pair, Parser};
use std::convert::TryFrom;
use std::fs;
use std::path::Path;

#[derive(pest_derive::Parser)]
#[grammar = "parse/grammar.pest"]
struct CrabParser;

pub fn parse(source: &Path) -> Result<CrabAst> {
    let source = fs::read_to_string(source)?;
    let parsed = CrabParser::parse(Rule::program, &source)?;
    trace!("Parsed AST is: {:#?}", parsed);
    // // There can only be one
    return match parsed.peek() {
        None => Err(ParseError::NoMatch),
        Some(pair) => CrabAst::try_from(pair),
    };
}

#[derive(Debug)]
pub struct CrabAst {
    pub functions: Vec<Func>,
}

#[derive(Debug)]
pub struct Func {
    pub signature: FuncSignature,
    pub body: CodeBlock,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub name: Ident,
    pub return_type: CrabType,
}

pub type Ident = String;

#[derive(Debug)]
pub struct CodeBlock {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    RETURN(Option<Expression>),
    ASSIGNMENT(Assignment),
    REASSIGNMENT(Assignment),
}

#[derive(Debug)]
pub struct Assignment {
    pub var_name: Ident,
    pub expression: Expression,
}

#[derive(Debug)]
pub struct FnCall {
    pub name: Ident,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Expression {
    PRIM(Primitive),
    FN_CALL(FnCall),
    VARIABLE(Ident),
}

#[derive(Debug)]
pub enum Primitive {
    UINT(u64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CrabType {
    UINT,
    VOID,
}

pub trait AstNode {
    // Build an instance of self from the given pair, assuming that the pair's rule type is correct
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized;
    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
}

/*
 * Generates a try-from function for getting the given crabNode from the specified rule type
 * The crabNode must implement NodeFromPair for the given rule type
 *
 * The first form only matches the given rule
 * The second form matches the given rule and ignores the second supplied rule
 * TODO: The third form matches the given rule and ignores all subsequent rules
 */
macro_rules! try_from_pair {
    ($crabNode:ty, $rule:path) => {
        impl TryFrom<Pair<'_, Rule>> for $crabNode {
            type Error = ParseError;
            fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
                match pair.as_rule() {
                    $rule => <$crabNode>::from_pair(pair),
                    _ => Err(ParseError::IncorrectRule(
                        String::from(stringify!($crabNode)),
                        String::from(stringify!($rule)),
                        format!("{:?}", pair.as_rule()),
                    )),
                }
            }
        }
    };

    ($crabNode:ty, $rule:path, $ig_rule:path) => {
        impl TryFrom<Pair<'_, Rule>> for $crabNode {
            type Error = ParseError;
            fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
                match pair.as_rule() {
                    $rule => <$crabNode>::from_pair(pair),
                    $ig_rule => Err(ParseError::None),
                    _ => Err(ParseError::IncorrectRule(
                        String::from(stringify!($crabNode)),
                        String::from(stringify!($rule)),
                        format!("{:?}", pair.as_rule()),
                    )),
                }
            }
        }
    };
}

macro_rules! visit_fns {
    ($node:ty) => {
        paste::item! {
            fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< pre_visit_ $node >](self)
            }

            fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< visit_ $node >](self)
            }

            fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< post_visit_ $node >](self)
            }
        }
    };
}

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut functions = vec![];
        for function in pair.into_inner() {
            match Func::try_from(function) {
                Ok(func) => functions.push(func),
                Err(err) => {
                    match err {
                        ParseError::None => {} // Do nothing
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(CrabAst { functions })
    }

    visit_fns!(CrabAst);
}

try_from_pair!(Func, Rule::function, Rule::EOI);
impl AstNode for Func {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let sig_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let signature = FuncSignature::try_from(sig_pair)?;
        let body_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let body = CodeBlock::try_from(body_pair)?;

        Ok(Func { signature, body })
    }

    visit_fns!(Func);
}

try_from_pair!(FuncSignature, Rule::fn_signature);
impl AstNode for FuncSignature {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let return_type = match inner.next() {
            Some(return_pair) => CrabType::try_from(return_pair)?,
            None => CrabType::VOID,
        };

        Ok(FuncSignature { name, return_type })
    }

    visit_fns!(FuncSignature);
}

try_from_pair!(CodeBlock, Rule::code_block);
impl AstNode for CodeBlock {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut statements = vec![];
        for statement in pair.into_inner() {
            match Statement::try_from(statement) {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    match err {
                        ParseError::None => {} // Do nothing
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(CodeBlock { statements })
    }

    visit_fns!(CodeBlock);
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
            _ => Err(ParseError::NoMatch),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN(ret) => visitor.pre_visit_Statement_RETURN(ret)?,
            Self::ASSIGNMENT(ass) => visitor.pre_visit_Statement_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.pre_visit_Statement_REASSIGNMENT(reass)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN(ret) => visitor.visit_Statement_RETURN(ret)?,
            Self::ASSIGNMENT(ass) => visitor.visit_Statement_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.visit_Statement_REASSIGNMENT(reass)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN(ret) => visitor.post_visit_Statement_RETURN(ret)?,
            Self::ASSIGNMENT(ass) => visitor.post_visit_Statement_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.post_visit_Statement_REASSIGNMENT(reass)?,
            _ => unimplemented!(),
        }
        Ok(())
    }
}

try_from_pair!(Expression, Rule::expression);
#[allow(unreachable_patterns)]
impl AstNode for Expression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::primitive => Ok(Expression::PRIM(Primitive::try_from(expr_type)?)),
            Rule::fn_call => Ok(Expression::FN_CALL(FnCall::try_from(expr_type)?)),
            Rule::ident => Ok(Expression::VARIABLE(Ident::from(expr_type.as_str()))),
            _ => Err(ParseError::NoMatch),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.pre_visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.pre_visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.pre_visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.post_visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.post_visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.post_visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }
}

try_from_pair!(Primitive, Rule::primitive);
#[allow(unreachable_patterns)]
impl AstNode for Primitive {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let prim_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match prim_type.as_rule() {
            Rule::uint64_primitive => Ok(Primitive::UINT(prim_type.as_str().parse()?)),
            _ => Err(ParseError::NoMatch),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.pre_visit_Primitive_UINT64(value)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.visit_Primitive_UINT64(value)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.post_visit_Primitive_UINT64(value)?,
            _ => unimplemented!(),
        }
        Ok(())
    }
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        match pair.as_str() {
            "uint" => Ok(Self::UINT),
            s => Err(ParseError::InvalidCrabType(String::from(s))),
        }
    }

    visit_fns!(CrabType);
}

try_from_pair!(FnCall, Rule::fn_call);
impl AstNode for FnCall {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        Ok(Self { name })
    }

    visit_fns!(FnCall);
}

/// Assignment requries a custom TryFrom implementation because it can be built from two different rules
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
        let expression = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self {
            var_name,
            expression,
        })
    }

    visit_fns!(Assignment);
}
