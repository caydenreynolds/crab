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

#[derive(Debug)]
pub struct FuncSignature {
    pub name: Ident,
}

pub type Ident = String;

#[derive(Debug)]
pub struct CodeBlock {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    RETURN(Option<Expression>),
}

#[derive(Debug)]
pub enum Expression {
    PRIM(Primitive),
}

#[derive(Debug)]
pub enum Primitive {
    UINT64(u64),
}

pub trait AstNode {
    // Build an instance of self from the given pair, assuming that the pair's rule type is correct
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized;
    fn visit(&self, visitor: &mut dyn AstVisitor);
    fn post_visit(&self, visitor: &mut dyn AstVisitor);
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
            fn visit(&self, visitor: &mut dyn AstVisitor) {
                visitor.[< visit_ $node >](self);
            }

            fn post_visit(&self, visitor: &mut dyn AstVisitor) {
                visitor.[< post_visit_ $node >](self);
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

        Ok(FuncSignature { name })
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
            _ => Err(ParseError::NoMatch),
        };
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::RETURN(ret) => visitor.visit_Statement_RETURN(ret),
            _ => unimplemented!(),
        }
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::RETURN(ret) => visitor.post_visit_Statement_RETURN(ret),
            _ => unimplemented!(),
        }
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
            _ => Err(ParseError::NoMatch),
        };
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::PRIM(prim) => visitor.visit_Expression_PRIM(prim),
            _ => unimplemented!(),
        }
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::PRIM(prim) => visitor.post_visit_Expression_PRIM(prim),
            _ => unimplemented!(),
        }
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
            Rule::uint64_primitive => Ok(Primitive::UINT64(prim_type.as_str().parse()?)),
            _ => Err(ParseError::NoMatch),
        };
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::UINT64(value) => visitor.visit_Primitive_UINT64(value),
            _ => unimplemented!(),
        }
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) {
        match self {
            Self::UINT64(value) => visitor.post_visit_Primitive_UINT64(value),
            _ => unimplemented!(),
        }
    }
}
