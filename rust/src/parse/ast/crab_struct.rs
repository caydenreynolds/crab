use crate::compile::CompileError;
use crate::parse::ast::{AstNode, CrabType, Ident, StructField};
use crate::parse::{ParseError, Rule};
use crate::try_from_pair;
use crate::{compile, parse};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
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
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );
        let mut fields = vec![];

        for in_pair in inner {
            fields.push(StructField::try_from(in_pair)?);
        }

        Ok(Self { name, fields })
    }
}
impl Struct {
    pub fn get_fields_as_basic_type<'a, 'ctx>(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> compile::Result<Vec<BasicTypeEnum<'ctx>>> {
        let mut btev = vec![];

        for field in &self.fields {
            btev.push(field.clone().crab_type.try_as_basic_type(context, module)?);
        }

        Ok(btev)
    }

    pub fn get_field_index(&self, name: &Ident) -> compile::Result<usize> {
        for i in 0..self.fields.len() {
            if &self.fields.get(i).unwrap().name == name {
                return Ok(i);
            }
        }
        Err(CompileError::StructFieldName(
            self.name.clone(),
            name.clone(),
        ))
    }

    pub fn get_field_crab_type(&self, name: &Ident) -> compile::Result<CrabType> {
        for field in &self.fields {
            if &field.name == name {
                return Ok(field.crab_type.clone());
            }
        }
        Err(CompileError::StructFieldName(
            self.name.clone(),
            name.clone(),
        ))
    }
}
