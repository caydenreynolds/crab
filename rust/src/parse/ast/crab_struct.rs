use crate::parse;
use crate::parse::ast::{AstNode, Ident, CrabStructField, StructIdent};
use crate::parse::{ParseError, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;
use serde::__private::de::Content::String;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabStruct {
    pub id: StructIdent,
    pub body: StrctBodyType,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StrctBodyType {
    FIELDS(Vec<CrabStructField>),
    COMPILER_PROVIDED,
}

try_from_pair!(CrabStruct, Rule::crab_struct);
impl AstNode for CrabStruct {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let id = StructIdent::try_from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?)?;
        let body = match next.as_rule() {
            Rule::compiler_provided => StrctBodyType::COMPILER_PROVIDED,
            Rule::struct_field => StrctBodyType::FIELDS(vec![CrabStructField::try_from(next)?]),
            _ => return Err(ParseError::NoMatch(String::from("Struct::from_pair"))),
        };

        let body = match body {
            StrctBodyType::FIELDS(mut fields) => {
                for in_pair in inner {
                    fields.push(CrabStructField::try_from(in_pair)?);
                }
                StrctBodyType::FIELDS(fields)
            }
            StrctBodyType::COMPILER_PROVIDED => StrctBodyType::COMPILER_PROVIDED, // Do nothing
        };

        Ok(Self { id, body })
    }
}
