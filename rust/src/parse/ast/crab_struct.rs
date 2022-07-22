use crate::parse::ast::{AstNode, CrabType, Ident};
use crate::parse::{ParseError, Rule, Result};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabStruct {
    pub name: Ident,
    pub body: StructBody,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StructBody {
    FIELDS(Vec<StructField>),
    COMPILER_PROVIDED,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructField {
    pub name: Ident,
    pub crab_type: CrabType,
}

struct StructFields(Vec<StructField>);

try_from_pair!(CrabStruct, Rule::crab_struct);
impl AstNode for CrabStruct {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> where Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );

        let next = inner
            .next()
            .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?;
        let body = match next.clone().as_rule() {
            Rule::compiler_provided => StructBody::COMPILER_PROVIDED,
            Rule::struct_fields => StructBody::FIELDS(StructFields::try_from(next).0),
            _ => return Err(ParseError::NoMatch(String::from("Struct::from_pair"))),
        };

        Ok(Self { name, body })
    }
}

try_from_pair!(StructField, Rule::struct_field);
impl AstNode for StructField {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> where Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        Ok(Self { name, crab_type })
    }
}

try_from_pair!(StructFields, Rule::struct_fields);
impl AstNode for StructFields {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> where Self: Sized {
        Ok(Self(pair.into_inner().try_fold(vec![], |fields, field| {
            Ok(fields.finsert(StructField::try_from(field)?))
        })?))
    }
}
