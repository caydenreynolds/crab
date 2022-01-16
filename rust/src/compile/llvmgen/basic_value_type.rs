use crate::parse::CrabType;
use inkwell::values::{ArrayValue, CallSiteValue, IntValue, VectorValue};

pub enum BasicValueType<'ctx> {
    IntValue(IntValue<'ctx>, CrabType),
    CallValue(CallSiteValue<'ctx>, CrabType),
    StringPrimitiveValue(VectorValue<'ctx>, CrabType),
    StringValue(ArrayValue<'ctx>, CrabType),
    None,
}

#[allow(unreachable_patterns)]
impl<'ctx> BasicValueType<'ctx> {
    pub fn to_crab_type(&self) -> CrabType {
        match self {
            Self::None => CrabType::VOID,
            Self::IntValue(_, ct) => *ct,
            Self::CallValue(_, ct) => *ct,
            Self::StringValue(_, ct) => *ct,
            Self::StringPrimitiveValue(_, ct) => *ct,
            _ => unimplemented!(),
        }
    }
}
