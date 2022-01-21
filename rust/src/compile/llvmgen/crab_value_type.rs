use crate::parse::CrabType;
use inkwell::values::{ArrayValue, BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, IntValue, PointerValue, VectorValue};
use crate::compile::{CompileError, Result};

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

    pub fn new_none() -> Self {
        Self::new(LLVMValueEnum::None, CrabType::VOID)
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
            LLVMValueEnum::None => None,
        };
    }

    pub fn try_as_basic_metadata_value(&self) -> Result<BasicMetadataValueEnum<'ctx>> {
        Ok(BasicMetadataValueEnum::from(self.get_as_basic_value().ok_or(CompileError::InvalidArgType(String::from(stringify!(CrabType::VOID))))?))
    }
}
