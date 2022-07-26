use crate::parse::ast::{AstNode, CrabType, Ident, StructId};
use crate::parse::{ParseError, Result, Rule};
use crate::{compile, try_from_pair, util};
use pest::iterators::Pair;
use std::convert::TryFrom;
use util::ListFunctional;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CrabStruct {
    pub id: StructId,
    pub body: StructBody,
}

try_from_pair!(CrabStruct, Rule::crab_struct);
impl AstNode for CrabStruct {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = StructId::try_from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
        )?;
        let body = StructBody::try_from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?,
        )?;

        Ok(Self { id: name, body })
    }
}
impl CrabStruct {
    /// Consumes self, returning a CrabStruct with the structId types resolved according to the
    /// given slice of CrabTypes
    pub fn resolve(self, types: &[CrabType]) -> compile::Result<Self> {
        Ok(Self {
            id: self.id.resolve(types)?,
            ..self
        })
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
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let next = pair.into_inner().next().ok_or(ParseError::ExpectedInner)?;
        Ok(match next.as_rule() {
            Rule::compiler_provided => StructBody::COMPILER_PROVIDED,
            Rule::struct_fields => StructBody::FIELDS(StructFields::try_from(next)?.0),
            _ => {
                return Err(ParseError::IncorrectRule(
                    String::from(stringify!(StructBody)),
                    format!("{:?} or {:?}", Rule::compiler_provided, Rule::struct_fields),
                    format!("{:?}", next.as_rule()),
                ))
            }
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
