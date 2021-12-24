use std::fs;
use std::path::Path;
use crate::parse::{ParseError, Result};
use pest::iterators::Pair;
use std::convert::TryFrom;
use log::trace;

#[derive(pest_derive::Parser)]
#[grammar = "parse/grammar.pest"]
struct SimpleParser;

pub fn parse(source: &Path) -> Result<SimpleAst> {
    unimplemented!();
    // let source = fs::read_to_string(source)?;
    // let parsed = CrabParser::parse(Rule::program, &source)?;
    // trace!("Parsed AST is: {:#?}", parsed);
    // // There can only be one
    // return match parsed.peek() {
    //     None => Err(ParseError::NoMatch),
    //     Some(pair) => SimpleAst::try_from(pair),
    // };
}

#[derive(Debug)]
pub struct SimpleAst {

}

// #[derive(Debug)]
// pub struct Func {
//     pub signature: FuncSignature,
//     pub body: CodeBlock,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct FuncSignature {
//     pub name: Ident,
// }
//
// pub type Ident = String;
//
// #[derive(Debug)]
// pub struct CodeBlock {
//     pub statements: Vec<Statement>,
// }
//
// #[derive(Debug)]
// pub enum Statement {
//     Return(Expression),
// }
//
// #[derive(Debug)]
// pub enum Expression {
//     Prim(Primitive),
// }
//
// #[derive(Debug)]
// pub enum Primitive {
//     UINT(u64),
// }
//
// trait NodeFromPair {
//     // Build an instance of self from the given pair, assuming that the pair's rule type is correct
//     fn from_pair(pair: Pair<Rule>) -> Result<Self>
//         where
//             Self: Sized;
// }
//
// /*
//  * Generates a try-from function for getting the given crabNode from the specified rule type
//  * The crabNode must implement NodeFromPair for the given rule type
//  *
//  * The first form only matches the given rule
//  * The second form matches the given rule and ignores the second supplied rule
//  * TODO: The third form matches the given rule and ignores all subsequent rules
//  */
// macro_rules! try_from_pair {
//     ($crabNode:ty, $rule:path) => {
//         impl TryFrom<Pair<'_, Rule>> for $crabNode {
//             type Error = ParseError;
//             fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
//                 match pair.as_rule() {
//                     $rule => <$crabNode>::from_pair(pair),
//                     _ => Err(ParseError::IncorrectRule(
//                         String::from(stringify!($crabNode)),
//                         String::from(stringify!($rule)),
//                         format!("{:?}", pair.as_rule()),
//                     )),
//                 }
//             }
//         }
//     };
//
//     ($crabNode:ty, $rule:path, $ig_rule:path) => {
//         impl TryFrom<Pair<'_, Rule>> for $crabNode {
//             type Error = ParseError;
//             fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
//                 match pair.as_rule() {
//                     $rule => <$crabNode>::from_pair(pair),
//                     $ig_rule => Err(ParseError::None),
//                     _ => Err(ParseError::IncorrectRule(
//                         String::from(stringify!($crabNode)),
//                         String::from(stringify!($rule)),
//                         format!("{:?}", pair.as_rule()),
//                     )),
//                 }
//             }
//         }
//     };
// }
//
// try_from_pair!(SimpleAst, Rule::program);
// impl NodeFromPair for SimpleAst {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         let mut functions = vec![];
//         for function in pair.into_inner() {
//             match Func::try_from(function) {
//                 Ok(func) => functions.push(func),
//                 Err(err) => {
//                     match err {
//                         ParseError::None => {} // Do nothing
//                         _ => return Err(err),
//                     }
//                 }
//             }
//         }
//         Ok(CrabAst {
//             functions,
//         })
//     }
// }
//
// try_from_pair!(Func, Rule::func, Rule::EOI);
// impl NodeFromPair for Func {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         let mut inner = pair.into_inner();
//         let sig_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         let signature = FuncSignature::try_from(sig_pair)?;
//         let body_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         let body = CodeBlock::try_from(body_pair)?;
//
//         Ok(Func { signature, body })
//     }
// }
//
// try_from_pair!(FuncSignature, Rule::fn_signature);
// impl NodeFromPair for FuncSignature {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         let mut inner = pair.into_inner();
//         let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
//
//         Ok(FuncSignature {
//             name,
//         })
//     }
// }
//
// try_from_pair!(CodeBlock, Rule::code_block);
// impl NodeFromPair for CodeBlock {
//     fn from_pair(_pair: Pair<Rule>) -> Result<Self> {
//         let mut statements = vec![];
//         for statement in pair.into_inner() {
//             match Statement::try_from(statement) {
//                 Ok(stmt) => statements.push(stmt),
//                 Err(err) => {
//                     match err {
//                         ParseError::None => {} // Do nothing
//                         _ => return Err(err),
//                     }
//                 }
//             }
//         }
//         Ok(CodeBlock {
//             statements,
//         })
//     }
// }
//
// try_from_pair!(Statement, Rule::statement);
// impl NodeFromPair for Statement {
//     fn from_pair(_pair: Pair<Rule>) -> Result<Self> {
//         Ok(Statement::Return(Expression::Prim(Primitive::UINT(0))))
//     }
// }
