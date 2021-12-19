// use crate::parse::{ParseError, Result};
// use log::trace;
// use pest::{iterators::Pair, Parser};
// use serde::{Deserialize, Serialize};
// use std::convert::TryFrom;
// use std::fs;
// use std::path::Path;
//
// #[derive(pest_derive::Parser)]
// #[grammar = "parse/grammar.pest"]
// struct CrabParser;
//
// pub fn parse(source: &Path) -> Result<CrabAst> {
//     let source = fs::read_to_string(source)?;
//     let parsed = CrabParser::parse(Rule::program, &source)?;
//     trace!("Parsed AST is: {:#?}", parsed);
//     // There can only be one
//     return match parsed.peek() {
//         None => Err(ParseError::NoMatch),
//         Some(pair) => CrabAst::try_from(pair),
//     };
// }

// #[derive(Debug)]
// pub struct CrabAst {
//     pub functions: Vec<Func>,
// }
//
// #[derive(Debug)]
// pub struct Func {
//     pub signature: FuncSignature,
//     pub body: CodeBlock,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct FuncSignature {
//     pub access_modifier: AccessModifier,
//     pub name: Ident,
//     pub args: Vec<TypedIdent>,
//     pub returns: Vec<CrabType>,
//     pub errable: bool,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct TypedIdent {
//     pub typed: CrabType,
//     pub name: Ident,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct CrabType {
//     pub name: Ident,
//     pub nullable: bool,
//     pub reference: bool,
// }
//
// pub type Ident = String;
//
// #[derive(Debug, Serialize, Deserialize)]
// pub enum AccessModifier {
//     Private,
//     Public,
//     Protected,
// }
//
// //TODO: Implement
// #[derive(Debug)]
// pub struct CodeBlock;
//
// trait NodeFromPair {
//     // Build an instance of self from the given pair, assuming that the pair's rule type is correct
//     fn from_pair(pair: Pair<Rule>) -> Result<Self>
//     where
//         Self: Sized;
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
// try_from_pair!(CrabAst, Rule::program);
// impl NodeFromPair for CrabAst {
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
//             functions: functions,
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
//         let access_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         let access_modifier = AccessModifier::try_from(access_pair)?;
//         let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
//
//         let mut args = vec![];
//         let args_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         for arg_pair in args_pair.into_inner() {
//             args.push(TypedIdent::try_from(arg_pair)?);
//         }
//
//         let mut returns = vec![];
//         let mut errable = false;
//         if let Some(returns_pair) = inner.next() {
//             for return_pair in returns_pair.into_inner() {
//                 match CrabType::try_from(return_pair.clone()) {
//                     Ok(crab_type) => returns.push(crab_type),
//                     Err(err) => match return_pair.as_rule() {
//                         Rule::errable => errable = true,
//                         _ => return Err(err),
//                     },
//                 }
//             }
//         }
//
//         Ok(FuncSignature {
//             access_modifier,
//             name,
//             args,
//             returns,
//             errable,
//         })
//     }
// }
//
// //TODO: implement
// try_from_pair!(CodeBlock, Rule::code_block);
// impl NodeFromPair for CodeBlock {
//     fn from_pair(_pair: Pair<Rule>) -> Result<Self> {
//         Ok(CodeBlock {})
//     }
// }
//
// try_from_pair!(AccessModifier, Rule::access_modifier);
// impl NodeFromPair for AccessModifier {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         return match pair.as_str() {
//             "" => Ok(AccessModifier::Private),
//             "pub" => Ok(AccessModifier::Public),
//             "pro" => Ok(AccessModifier::Protected),
//             _ => Err(ParseError::BadAccessModifier),
//         };
//     }
// }
//
// try_from_pair!(TypedIdent, Rule::typed_ident);
// impl NodeFromPair for TypedIdent {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         let mut inner = pair.into_inner();
//         let name_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         let name = Ident::from(name_pair.as_str());
//         let typed_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
//         let typed = CrabType::try_from(typed_pair)?;
//         return Ok(TypedIdent { typed, name });
//     }
// }
//
// try_from_pair!(CrabType, Rule::crab_type);
// impl NodeFromPair for CrabType {
//     fn from_pair(pair: Pair<Rule>) -> Result<Self> {
//         let mut reference = false;
//         let mut nullable = false;
//         let mut name = Ident::from("");
//         for inner_pair in pair.into_inner() {
//             match inner_pair.clone().as_rule() {
//                 Rule::reference => reference = true,
//                 Rule::ident => name = Ident::from(inner_pair.as_str()),
//                 Rule::nullable => nullable = true,
//                 _ => {
//                     return Err(ParseError::IncorrectRule(
//                         String::from(stringify!(CrabType)),
//                         format!(
//                             "One of [{}, {}, {}]",
//                             stringify!(Rule::reference),
//                             stringify!(Rule::ident),
//                             stringify!(Rule::nullable)
//                         ),
//                         String::from(stringify!(inner_pair.as_rule())),
//                     ))
//                 }
//             }
//         }
//
//         Ok(CrabType {
//             name,
//             nullable,
//             reference,
//         })
//     }
// }
