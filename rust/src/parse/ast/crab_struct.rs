use std::convert::TryFrom;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use pest::iterators::Pair;
use crate::parse::ast::{AstNode, CrabType, Expression, Ident};
use crate::parse::{Rule, ParseError};
use crate::try_from_pair;
use crate::{parse, compile};

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<StructField>,
}

try_from_pair!(Struct, Rule::crab_struct);
impl AstNode for Struct {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?.as_str());
        let mut fields = vec![];

        for in_pair in inner {
            fields.push(StructField::try_from(in_pair)?);
        }

        Ok(Self { name, fields })
    }
}
impl Struct {
    pub fn get_fields_as_basic_type<'a, 'ctx>(&self, context: &'ctx Context, module: &'a Module<'ctx>) -> compile::Result<Vec<BasicTypeEnum<'ctx>>> {
        let mut btev = vec![];

        for field in &self.fields {
            btev.push(field.clone().crab_type.try_as_basic_type(context, module)?);
        }

        Ok(btev)
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: Ident,
    pub crab_type: CrabType,
}

try_from_pair!(StructField, Rule::struct_field);
impl AstNode for StructField {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        Ok(Self { name, crab_type })
    }
}

#[derive(Debug, Clone)]
pub struct StructInit {
    pub name: Ident,
    pub fields: Vec<StructFieldInit>,
}

try_from_pair!(StructInit, Rule::struct_init);
impl AstNode for StructInit {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?.as_str());
        let mut fields = vec![];

        for in_pair in inner {
            fields.push(StructFieldInit::try_from(in_pair)?);
        }

        Ok(Self { name, fields })
    }
}

#[derive(Debug, Clone)]
pub struct StructFieldInit {
    pub name: Ident,
    pub value: Expression,
}

try_from_pair!(StructFieldInit, Rule::struct_field_init);
impl AstNode for StructFieldInit {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let value = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { name, value })
    }
}
