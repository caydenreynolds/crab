use crate::parse::ast::{AstNode, CrabType, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::{try_from_pair, util};
use pest::iterators::Pair;
use std::convert::TryFrom;
use util::ListFunctional;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabStruct {
    pub name: Ident,
    pub body: StructBody,
}

try_from_pair!(CrabStruct, Rule::crab_struct);
impl AstNode for CrabStruct {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );
        let body = StructBody::try_from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?)?;

        Ok(Self { name, body })
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StructBody {
    FIELDS(Vec<StructField>),
    COMPILER_PROVIDED,
}
try_from_pair!(StructBody, Rule::struct_body);
impl AstNode for StructBody {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> where Self: Sized {
        Ok(match pair.as_rule() {
            Rule::compiler_provided => StructBody::COMPILER_PROVIDED,
            Rule::struct_fields => StructBody::FIELDS(StructFields::try_from(pair)?.0),
            _ => return Err(ParseError::IncorrectRule(
                String::from(stringify!(StructBody)),
                format!("{:?} or {:?}", Rule::compiler_provided, Rule::struct_fields),
                format!("{:?}", pair.as_rule()),
            )),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructField {
    pub name: Ident,
    pub crab_type: CrabType,
}
try_from_pair!(StructField, Rule::struct_field);
impl AstNode for StructField {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        Ok(Self { name, crab_type })
    }
}

struct StructFields(Vec<StructField>);
try_from_pair!(StructFields, Rule::struct_fields);
impl AstNode for StructFields {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |fields, field| {
                Result::Ok(fields.fpush(StructField::try_from(field)?))
            })?,
        ))
    }
}
