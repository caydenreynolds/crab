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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    UINT,
    VOID,
    STRING,
    FLOAT,
    BOOL,
    STRUCT(Ident),
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        match pair.as_str() {
            "__uint64__" => Ok(Self::UINT),
            "String" => Ok(Self::STRING),
            "Float" => Ok(Self::FLOAT),
            "Bool" => Ok(Self::BOOL),
            s => Ok(Self::STRUCT(Ident::from(s))),
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
            Self::UINT => Ok(AnyTypeEnum::IntType(context.i64_type())),
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
        };
    }

    pub fn try_as_basic_type(
        &self,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> compile::Result<BasicTypeEnum<'ctx>> {
        return match self {
            Self::UINT => Ok(BasicTypeEnum::IntType(context.i64_type())),
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
            Self::UINT => Ok(context.i64_type().fn_type(param_types, variadic)),
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
        };
    }
}
