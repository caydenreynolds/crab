use inkwell::values::IntValue;

pub enum BasicValueType<'ctx> {
    IntType(IntValue<'ctx>),
    None,
}
