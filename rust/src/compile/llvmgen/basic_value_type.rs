use crate::parse::CrabType;
use inkwell::values::IntValue;

pub enum BasicValueType<'ctx> {
    IntType(IntValue<'ctx>, CrabType),
    None,
}

impl<'ctx> BasicValueType<'ctx> {
    pub(crate) fn to_crab_type(&self) -> CrabType {
        match self {
            Self::None => CrabType::VOID,
            Self::IntType(_, ct) => *ct,
        }
    }
}
