use crate::parse::CrabType;
use inkwell::values::{CallSiteValue, IntValue};

pub enum BasicValueType<'ctx> {
    IntType(IntValue<'ctx>, CrabType),
    CallValue(CallSiteValue<'ctx>, CrabType),
    None,
}

#[allow(unreachable_patterns)]
impl<'ctx> BasicValueType<'ctx> {
    pub(crate) fn to_crab_type(&self) -> CrabType {
        match self {
            Self::None => CrabType::VOID,
            Self::IntType(_, ct) => *ct,
            Self::CallValue(_, ct) => *ct,
            _ => unimplemented!(),
        }
    }
}
