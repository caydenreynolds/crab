use crate::compile::{CompileError, Result};
use crate::parse::{CrabType, Ident};
use inkwell::values::{
    ArrayValue, BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FloatValue, IntValue,
    PointerValue, StructValue, VectorValue,
};

#[derive(Clone)]
pub struct CrabValueType<'ctx> {
    llvm_value: LLVMValueEnum<'ctx>,
    crab_type: CrabType,
}

#[derive(Clone)]
pub enum LLVMValueEnum<'ctx> {
    IntValue(IntValue<'ctx>),
    ArrayValue(ArrayValue<'ctx>),
    CallSiteValue(CallSiteValue<'ctx>),
    PointerValue(PointerValue<'ctx>),
    VectorValue(VectorValue<'ctx>),
    FloatValue(FloatValue<'ctx>),
    StructValue(StructValue<'ctx>),
    None,
}

#[allow(unreachable_patterns)]
impl<'ctx> CrabValueType<'ctx> {
    pub fn new(llvm_value: LLVMValueEnum<'ctx>, crab_type: CrabType) -> Self {
        Self {
            llvm_value,
            crab_type,
        }
    }

    pub fn new_string(ptr: PointerValue<'ctx>) -> Self {
        Self::new(LLVMValueEnum::PointerValue(ptr), CrabType::STRING)
    }

    pub fn new_uint(uint: IntValue<'ctx>) -> Self {
        Self::new(LLVMValueEnum::IntValue(uint), CrabType::UINT)
    }

    pub fn new_call_value(value: CallSiteValue<'ctx>, ct: CrabType) -> Self {
        Self::new(LLVMValueEnum::CallSiteValue(value), ct)
    }

    pub fn new_ptr(value: PointerValue<'ctx>, ct: CrabType) -> Self {
        Self::new(LLVMValueEnum::PointerValue(value), ct)
    }

    pub fn new_bool(val: IntValue<'ctx>) -> Self {
        Self::new(LLVMValueEnum::IntValue(val), CrabType::BOOL)
    }

    pub fn new_struct(val: StructValue<'ctx>, name: Ident) -> Self {
        Self::new(LLVMValueEnum::StructValue(val), CrabType::STRUCT(name))
    }

    pub fn new_void() -> Self {
        Self::new(LLVMValueEnum::None, CrabType::VOID)
    }

    pub fn from_basic_value_enum(bve: BasicValueEnum<'ctx>, ct: CrabType) -> Self {
        let llvm_value = match bve {
            BasicValueEnum::ArrayValue(val) => LLVMValueEnum::ArrayValue(val),
            BasicValueEnum::PointerValue(val) => LLVMValueEnum::PointerValue(val),
            BasicValueEnum::VectorValue(val) => LLVMValueEnum::VectorValue(val),
            BasicValueEnum::IntValue(val) => LLVMValueEnum::IntValue(val),
            BasicValueEnum::FloatValue(val) => LLVMValueEnum::FloatValue(val),
            BasicValueEnum::StructValue(val) => LLVMValueEnum::StructValue(val),
        };
        Self::new(llvm_value, ct)
    }

    pub fn get_crab_type(&self) -> CrabType {
        self.crab_type
    }

    pub fn get_llvm_type(&self) -> &LLVMValueEnum<'ctx> {
        &self.llvm_value
    }

    pub fn get_as_basic_value(&self) -> Option<BasicValueEnum<'ctx>> {
        return match self.llvm_value {
            LLVMValueEnum::IntValue(v) => Some(BasicValueEnum::IntValue(v)),
            LLVMValueEnum::PointerValue(v) => Some(BasicValueEnum::PointerValue(v)),
            LLVMValueEnum::VectorValue(v) => Some(BasicValueEnum::VectorValue(v)),
            LLVMValueEnum::ArrayValue(v) => Some(BasicValueEnum::ArrayValue(v)),
            LLVMValueEnum::CallSiteValue(v) => Some(
                v.try_as_basic_value()
                    .expect_left("Expected function call to return a basic value"),
            ),
            LLVMValueEnum::FloatValue(v) => Some(BasicValueEnum::FloatValue(v)),
            LLVMValueEnum::StructValue(v) => Some(BasicValueEnum::StructValue(v)),
            LLVMValueEnum::None => None,
        };
    }

    pub fn try_as_basic_metadata_value(&self) -> Result<BasicMetadataValueEnum<'ctx>> {
        Ok(BasicMetadataValueEnum::from(
            self.get_as_basic_value()
                .ok_or(CompileError::InvalidArgType(String::from(stringify!(
                    CrabType::VOID
                ))))?,
        ))
    }

    pub fn try_as_ptr_value(&self) -> Result<PointerValue<'ctx>> {
        match self.llvm_value {
            LLVMValueEnum::PointerValue(val) => Ok(val),
            _ => Err(CompileError::VarValueType(String::from("PointerValue"))),
        }
    }

    pub fn try_as_bool_value(&self) -> Result<IntValue<'ctx>> {
        match self.crab_type {
            CrabType::BOOL => match self.llvm_value {
                LLVMValueEnum::IntValue(val) => Ok(val),
                _ => panic!("Reached an unreachable line in CrabValueType::try_as_bool_value()"),
            },
            _ => Err(CompileError::VarValueType(String::from("BoolValue"))),
        }
    }
}
