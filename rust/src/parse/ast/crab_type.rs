use crate::compile::CompileError;
use crate::parse::ast::{AstNode, FnParam, Ident};
use crate::parse::{ParseError, Rule};
use crate::{compile, parse, try_from_pair};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::AddressSpace;
use log::trace;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    UINT8,
    UINT64,
    STRING,
    VOID,
    FLOAT,
    BOOL,
    STRUCT(Ident),
    UINT8_ARRAY(u32),
    STRUCT_ARRAY(Ident, u32),
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        match inner.clone().count() {
            1 => {
                let ct = inner.next().ok_or(ParseError::ExpectedInner)?;
                match ct.as_str() {
                    "__uint64__" => Ok(Self::UINT64),
                    "__uint8__" => Ok(Self::UINT8),
                    "__string__" => Ok(Self::STRING),
                    "Float" => Ok(Self::FLOAT),
                    "Bool" => Ok(Self::BOOL),
                    s => Ok(Self::STRUCT(Ident::from(s))),
                }
            }
            2 => {
                let ct = inner.next().ok_or(ParseError::ExpectedInner)?;
                let array_len_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
                let array_len = array_len_pair.as_str().parse()?;

                match ct.as_str() {
                    "__uint8__" => Ok(Self::UINT8_ARRAY(array_len)),
                    "__uint64__" => Err(ParseError::InvalidCrabType(String::from(
                        "__uint64__ array",
                    ))),
                    "Float" => Err(ParseError::InvalidCrabType(String::from("Float array"))),
                    "Bool" => Err(ParseError::InvalidCrabType(String::from("Bool array"))),
                    "Void" => Err(ParseError::InvalidCrabType(String::from("Void array"))),
                    s => Ok(Self::STRUCT_ARRAY(Ident::from(s), array_len)),
                }
            }
            _ => unreachable!(),
        }
    }
}
impl<'a, 'ctx> CrabType {
    pub fn try_as_llvm_type(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> compile::Result<AnyTypeEnum<'ctx>> {
        return match self {
            Self::UINT64 => Ok(AnyTypeEnum::IntType(context.i64_type())),
            Self::UINT8 => Ok(AnyTypeEnum::IntType(context.i8_type())),
            // TODO: Figure out what to do about address spaces
            Self::STRING => Ok(AnyTypeEnum::PointerType(
                context.i8_type().ptr_type(AddressSpace::Generic),
            )),
            Self::FLOAT => Ok(AnyTypeEnum::FloatType(context.f64_type())),
            Self::BOOL => Ok(AnyTypeEnum::IntType(context.custom_width_int_type(1))),
            Self::VOID => Ok(AnyTypeEnum::VoidType(context.void_type())),
            Self::STRUCT(id) => Ok(AnyTypeEnum::PointerType(
                module
                    .get_struct_type(id)
                    .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                    .ptr_type(AddressSpace::Generic),
            )),
            Self::UINT8_ARRAY(len) => {
                Ok(AnyTypeEnum::ArrayType(context.i8_type().array_type(*len)))
            }
            Self::STRUCT_ARRAY(id, len) => Ok(AnyTypeEnum::ArrayType(
                module
                    .get_struct_type(id)
                    .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                    .ptr_type(AddressSpace::Generic)
                    .array_type(*len),
            )),
        };
    }

    pub fn try_as_basic_type(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> compile::Result<BasicTypeEnum<'ctx>> {
        return match self {
            Self::UINT64 => Ok(BasicTypeEnum::IntType(context.i64_type())),
            Self::UINT8 => Ok(BasicTypeEnum::IntType(context.i8_type())),
            // TODO: Figure out what to do about address spaces
            Self::STRING => Ok(BasicTypeEnum::PointerType(
                context.i8_type().ptr_type(AddressSpace::Generic),
            )),
            Self::FLOAT => Ok(BasicTypeEnum::FloatType(context.f64_type())),
            Self::BOOL => Ok(BasicTypeEnum::IntType(context.custom_width_int_type(1))),
            Self::STRUCT(id) => Ok(BasicTypeEnum::PointerType(
                module
                    .get_struct_type(id)
                    .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                    .ptr_type(AddressSpace::Generic),
            )),
            Self::VOID => Err(CompileError::InvalidArgType(String::from(stringify!(
                CrabType::Void
            )))),
            Self::UINT8_ARRAY(len) => {
                Ok(BasicTypeEnum::ArrayType(context.i8_type().array_type(*len)))
            }
            Self::STRUCT_ARRAY(id, len) => Ok(BasicTypeEnum::ArrayType(
                module
                    .get_struct_type(id)
                    .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                    .ptr_type(AddressSpace::Generic)
                    .array_type(*len),
            )),
        };
    }

    pub fn try_as_basic_metadata_type(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> compile::Result<BasicMetadataTypeEnum<'ctx>> {
        Ok(BasicMetadataTypeEnum::from(
            self.try_as_basic_type(context, module)?,
        ))
    }

    pub fn try_as_fn_type(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        args: &[FnParam],
        variadic: bool,
    ) -> compile::Result<FunctionType<'ctx>> {
        trace!("CrabType as fn_type");

        let mut param_vec = vec![];
        for ti in args {
            param_vec.push(ti.crab_type.try_as_basic_metadata_type(context, module)?);
        }
        let param_types = param_vec.as_slice();

        return match self {
            Self::UINT64 => Ok(context.i64_type().fn_type(param_types, variadic)),
            Self::UINT8 => Ok(context.i8_type().fn_type(param_types, variadic)),
            // TODO: Figure out what to do about address spaces
            Self::STRING => Ok(context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .fn_type(param_types, false)),
            Self::BOOL => Ok(context
                .custom_width_int_type(1)
                .fn_type(param_types, variadic)),
            Self::FLOAT => Ok(context.f64_type().fn_type(param_types, variadic)),
            Self::STRUCT(id) => Ok(module
                .get_struct_type(id)
                .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                .ptr_type(AddressSpace::Generic)
                .fn_type(param_types, variadic)),
            Self::VOID => Ok(context.void_type().fn_type(param_types, variadic)),
            Self::UINT8_ARRAY(len) => Ok(context
                .i8_type()
                .array_type(*len)
                .fn_type(param_types, variadic)),
            Self::STRUCT_ARRAY(id, len) => Ok(module
                .get_struct_type(id)
                .ok_or(CompileError::StructDoesNotExist(id.clone()))?
                .ptr_type(AddressSpace::Generic)
                .array_type(*len)
                .fn_type(param_types, variadic)),
        };
    }
}
