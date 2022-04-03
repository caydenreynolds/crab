use crate::compile::CompileError;
use crate::parse::ast::{AstNode, FnParam, Ident};
use crate::parse::{ParseError, Rule};
use crate::{compile, parse, try_from_pair};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::AddressSpace;
use log::trace;
use pest::iterators::Pair;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CrabType {
    UINT8,
    UINT64,
    STRING,
    VOID,
    FLOAT,
    BOOL,
    STRUCT(Ident), // TODO: Struct currently encompasses both structs and interfaces
    LIST(Box<CrabType>),
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        let next = pair.into_inner().next().ok_or(ParseError::ExpectedInner)?;
        match next.clone().as_rule() {
            Rule::type_name => match next.as_str() {
                "__uint64__" => Ok(Self::UINT64),
                "__uint8__" => Ok(Self::UINT8),
                "__string__" => Ok(Self::STRING),
                "Float" => Ok(Self::FLOAT),
                "Bool" => Ok(Self::BOOL),
                s => Ok(Self::STRUCT(Ident::from(s))),
            },
            Rule::crab_type => Ok(Self::LIST(Box::new(CrabType::try_from(next)?))),
            _ => Err(ParseError::NoMatch(String::from("CrabType::from_pair"))),
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
            Self::LIST(l) => Ok(AnyTypeEnum::PointerType(
                l.try_as_basic_type(context, module)?
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
            Self::LIST(l) => Ok(BasicTypeEnum::PointerType(
                l.try_as_basic_type(context, module)?
                    .ptr_type(AddressSpace::Generic),
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
            Self::LIST(l) => Ok(l
                .try_as_basic_type(context, module)?
                .ptr_type(AddressSpace::Generic)
                .fn_type(param_types, variadic)),
        };
    }

    pub fn try_get_struct_name(&self) -> crate::compile::Result<Ident> {
        match self {
            Self::STRUCT(id) => Ok(id.clone()),
            _ => Err(CompileError::NotAStruct),
        }
    }
}

impl Display for CrabType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrabType::UINT8 => write!(f, "UINT8"),
            CrabType::UINT64 => write!(f, "UINT64"),
            CrabType::STRING => write!(f, "STRING"),
            CrabType::VOID => write!(f, "VOID"),
            CrabType::FLOAT => write!(f, "FLOAT"),
            CrabType::BOOL => write!(f, "BOOL"),
            CrabType::STRUCT(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
        }?;

        Ok(())
    }
}
