use crate::parse::CrabType;
use inkwell::values::PointerValue;

pub struct VarValue<'ctx> {
    pub pointer: PointerValue<'ctx>,
    pub crab_type: CrabType,
}

impl<'ctx> VarValue<'ctx> {
    pub fn new(pointer: PointerValue<'ctx>, crab_type: CrabType) -> Self {
        Self { pointer, crab_type }
    }
}
