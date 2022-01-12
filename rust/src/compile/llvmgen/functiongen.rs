use std::collections::HashMap;
// use inkwell::basic_block::BasicBlock;
use crate::compile::BasicValueType;
use crate::compile::{CompileError, Result};
use crate::parse::{CrabType, Ident};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{CallSiteValue, IntValue, PointerValue};
use log::trace;

pub struct Functiongen<'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    variables: HashMap<Ident, PointerValue<'ctx>>,
    //basic_block: BasicBlock<'ctx>,
}

impl<'ctx> Functiongen<'ctx> {
    pub fn new(
        name: &str,
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) -> Result<Functiongen<'ctx>> {
        trace!("Creating new functiongen for a function with name {}", name);
        let fn_value_opt = module.get_function(name);
        match fn_value_opt {
            Some(fn_value) => {
                let basic_blocks = fn_value.get_basic_blocks();
                let basic_block = basic_blocks.get(0).unwrap();
                let builder = context.create_builder();
                builder.position_at_end(*basic_block);
                let variables = HashMap::new();
                Ok(Self {
                    builder,
                    context,
                    variables,
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
        trace!("Building return statement");
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
        trace!("Building constant u64");
        self.context.i64_type().const_int(value, false)
    }

    pub fn build_fn_call(
        &mut self,
        fn_name: &Ident,
        module: &Module<'ctx>,
    ) -> Result<CallSiteValue<'ctx>> {
        trace!("Building a call to function {}", fn_name);
        let fn_value_opt = module.get_function(fn_name);
        match fn_value_opt {
            Some(fn_value) => Ok(self.builder.build_call(fn_value, &[], "call")),
            None => Err(CompileError::CouldNotFindFunction(String::from(fn_name))),
        }
    }

    pub fn build_assignment(&mut self, name: &Ident, value: &BasicValueType) -> Result<()> {
        trace!("Building an assignment to variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        // Only support i64 constants for now, and error on attempting to mutate a variable
        let val_ptr = match val_ptr_result {
            None => self.builder.build_alloca(self.context.i64_type(), name),
            _ => return Err(CompileError::VarAlreadyExists(name.clone())),
        };
        self.variables.insert(name.clone(), val_ptr);
        //TODO: Type checking
        match value {
            BasicValueType::IntType(v, _) => self.builder.build_store(val_ptr, *v),
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn build_retrieve_var(&mut self, name: &Ident) -> Result<BasicValueType<'ctx>> {
        trace!("Retreiving a variable with name {}", name);
        match self.variables.get(name) {
            Some(ptr_val) => Ok(BasicValueType::IntType(
                self.builder.build_load(*ptr_val, name).into_int_value(),
                CrabType::UINT,
            )),
            None => Err(CompileError::NoVar(name.clone())),
        }
    }
}
