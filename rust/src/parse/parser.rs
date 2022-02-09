use crate::parse::{ParseError, Result};
use log::trace;
use pest::Parser;
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use crate::parse::ast::CrabAst;

#[derive(pest_derive::Parser)]
#[grammar = "parse/grammar.pest"]
struct CrabParser;

pub fn parse(source: &Path) -> Result<CrabAst> {
    let source = fs::read_to_string(source)?;
    let parsed = CrabParser::parse(Rule::program, &source)?;
    trace!("Parsed source is: {:#?}", parsed);
    // // There can only be one
    return match parsed.peek() {
        None => Err(ParseError::NoMatch(String::from("parse"))),
        Some(pair) => CrabAst::try_from(pair),
    };
}
