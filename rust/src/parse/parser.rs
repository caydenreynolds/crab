use crate::parse::ast::CrabAst;
use crate::parse::{ParseError, Result};
use log::debug;
use pest::Parser;
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(pest_derive::Parser)]
#[grammar = "parse/grammar.pest"]
struct CrabParser;

pub fn parse(sources: &[PathBuf]) -> Result<CrabAst> {
    let mut crab_ast = CrabAst::default();
    for source in sources {
        debug!("Parsing crabfile {:#?}", source);
        let ca = parse_file(source)?;
        crab_ast = crab_ast.join(ca);
    }
    crab_ast.verify()?;
    Ok(crab_ast)
}

pub fn parse_file(source: &Path) -> Result<CrabAst> {
    let source = fs::read_to_string(source)?;
    let parsed = CrabParser::parse(Rule::program, &source)?;
    // There can only be one
    return match parsed.peek() {
        None => Err(ParseError::NoMatch(String::from("parse"))),
        Some(pair) => CrabAst::try_from(pair),
    };
}
