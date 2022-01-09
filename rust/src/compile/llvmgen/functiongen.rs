// use inkwell::basic_block::BasicBlock;
use crate::compile::BasicValueType;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::IntValue;

pub struct Functiongen<'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    //basic_block: BasicBlock<'ctx>,
}

impl<'ctx> Functiongen<'ctx> {
    pub fn new(name: &str, context: &'ctx Context, module: &Module<'ctx>) -> Functiongen<'ctx> {
        let fn_type = context.i64_type().fn_type(&[], false);
        let fn_value = module.add_function(name, fn_type, None);
        let basic_block = context.append_basic_block(fn_value, "entry");
        let builder = context.create_builder();
        builder.position_at_end(basic_block);
        Functiongen {
            builder,
            context,
            //basic_block
        }
    }

    #[allow(unreachable_patterns)]
    pub fn build_return(&mut self, value: &BasicValueType<'ctx>) {
        match value {
            BasicValueType::IntType(value, _) => self.builder.build_return(Some(value)),
            BasicValueType::None => self.builder.build_return(None),
            _ => unimplemented!(),
        };
    }

    pub fn build_const_u64(&self, value: u64) -> IntValue<'ctx> {
        self.context.i64_type().const_int(value, false)
    }
}
