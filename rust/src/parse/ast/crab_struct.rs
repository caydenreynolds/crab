use crate::compile::CompileError;
use crate::parse::ast::{AstNode, CrabType, Expression, Func, Ident};
use crate::parse::{ParseError, Rule};
use crate::try_from_pair;
use crate::{compile, parse};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use pest::iterators::Pair;
use std::convert::TryFrom;

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
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );
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

#[derive(Debug, Clone)]
pub struct StructImpl {
    pub struct_name: Ident,
    pub interface_name: Option<Ident>,
    pub fns: Vec<Func>,
}

try_from_pair!(StructImpl, Rule::impl_block);
impl AstNode for StructImpl {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let next_opt = inner.peek();

        let interface_name = match next_opt {
            None => None,
            Some(next_pair) => match next_pair.clone().as_rule() {
                Rule::function => None,
                Rule::ident => {
                    inner.next();
                    Some(Ident::from(next_pair.as_str()))
                }
                rule => {
                    return Err(ParseError::IncorrectRule(
                        String::from("StructImpl"),
                        String::from("function or ident"),
                        format!("{:#?}", rule),
                    ))
                }
            },
        };

        let mut fns = vec![];
        for in_pair in inner {
            fns.push(Func::try_from(in_pair)?);
        }

        Ok(Self {
            struct_name,
            interface_name,
            fns,
        })
    }
}
