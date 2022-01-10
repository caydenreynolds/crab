// use inkwell::basic_block::BasicBlock;
use crate::compile::BasicValueType;
use crate::compile::{CompileError, Result};
use crate::parse::Ident;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{CallSiteValue, IntValue};

pub struct Functiongen<'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    // module: &'ctx Module<'ctx>,
    //basic_block: BasicBlock<'ctx>,
}

impl<'ctx> Functiongen<'ctx> {
    pub fn new(
        name: &str,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<Functiongen<'ctx>> {
        let fn_value_opt = module.get_function(name);
        match fn_value_opt {
            Some(fn_value) => {
                let basic_blocks = fn_value.get_basic_blocks();
                let basic_block = basic_blocks.get(0).unwrap();
                let builder = context.create_builder();
                builder.position_at_end(*basic_block);
                Ok(Self {
                    builder,
                    context,
                    // module,
                    //basic_block
                })
            }
            // This should never happen
            None => Err(CompileError::CouldNotFindFunction(String::from(name))),
        }
    }

    #[allow(unreachable_patterns)]
    pub fn build_return(&mut self, value: &BasicValueType<'ctx>) {
        match value {
            BasicValueType::IntType(value, _) => self.builder.build_return(Some(value)),
            BasicValueType::None => self.builder.build_return(None),
            BasicValueType::CallValue(value, _) => self.builder.build_return(Some(
                &value
                    .try_as_basic_value()
                    .expect_left("Idk what's going on here"),
            )),
            _ => unimplemented!(),
        };
    }

    pub fn build_const_u64(&self, value: u64) -> IntValue<'ctx> {
        self.context.i64_type().const_int(value, false)
    }

    pub fn build_fn_call(
        &mut self,
        fn_name: &Ident,
        module: &Module<'ctx>,
    ) -> Result<CallSiteValue<'ctx>> {
        let fn_value_opt = module.get_function(fn_name);
        match fn_value_opt {
            Some(fn_value) => Ok(self.builder.build_call(fn_value, &[], "call")),
            None => Err(CompileError::CouldNotFindFunction(String::from(fn_name))),
        }
    }
}
