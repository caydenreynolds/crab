use std::collections::HashMap;
use crate::parse::ast::{AstNode, CrabType, Ident, StructId};
use crate::parse::{ParseError, Result, Rule};
use crate::{compile, try_from_pair, util};
use pest::iterators::Pair;
use std::convert::TryFrom;
use util::ListFunctional;
use crate::compile::CompileError;

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
        let unresolved = self.id.clone();
        let resolved = self.id.resolve(types)?;
        let resolution_map = HashMap::from(
            unresolved
                .tmpls
                .into_iter()
                .zip(resolved.tmpls.clone().into_iter())
                .collect()
        );
        let resolved_body = self.body.resolve(resolution_map);
        Ok(Self {
            id: resolved,
            body: resolved_body,
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
impl StructBody {
    fn resolve(self, resolution_map: HashMap<StructId, StructId>) -> Self {
        match self {
            StructBody::COMPILER_PROVIDED => StructBody::COMPILER_PROVIDED,
            StructBody::FIELDS(fields) => {
                StructBody::FIELDS(
                    fields.into_iter().map(|field| {
                        match resolution_map.get(&field.crab_type.into()) {
                            Some(si) => si.into(),
                            None => field,
                        }
                    }).collect()
                )
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
