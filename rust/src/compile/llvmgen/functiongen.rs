use inkwell::AddressSpace;
use std::collections::HashMap;
// use inkwell::basic_block::BasicBlock;
use crate::compile::llvmgen::VarValue;
use crate::compile::CrabValueType;
use crate::compile::{CompileError, Result};
use crate::parse::{CrabType, Ident};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::CallSiteValue;
use log::trace;
use uuid::Uuid;

pub struct Functiongen<'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    variables: HashMap<Ident, VarValue<'ctx>>,
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
    pub fn build_return(&mut self, value: &CrabValueType<'ctx>) {
        trace!("Building return statement");
        // Have to do this match block for reasons. Surprised Rust doesn't have a function that just handles this.
        match value.get_as_basic_value() {
            Some(x) => self.builder.build_return(Some(&x)),
            None => self.builder.build_return(None),
        };
    }

    pub fn build_const_u64(&self, value: u64) -> CrabValueType<'ctx> {
        trace!("Building constant u64");
        CrabValueType::new_uint(self.context.i64_type().const_int(value, false))
    }

    pub fn build_const_string(&self, value: &String) -> Result<CrabValueType<'ctx>> {
        trace!("Building constant string");
        let str_ptr = self
            .builder
            .build_global_string_ptr(&value, &Uuid::new_v4().to_string())
            .as_pointer_value();
        Ok(CrabValueType::new_string(str_ptr))
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

    pub fn build_create_var(&mut self, name: &Ident, expr_type: CrabType) -> Result<()> {
        trace!("Creating a new variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        // Only support i64 constants for now, and error on attempting to mutate a variable
        let val_ptr = match val_ptr_result {
            None => match expr_type {
                CrabType::UINT => self.builder.build_alloca(self.context.i64_type(), name),
                // CrabType::STRING(size) => self.builder.build_alloca(self.context.i8_type().array_type(size).ptr_type(AddressSpace::Generic), name),
                CrabType::STRING => self
                    .builder
                    .build_alloca(self.context.i8_type().ptr_type(AddressSpace::Generic), name),
                _ => unimplemented!(),
            },
            _ => return Err(CompileError::VarAlreadyExists(name.clone())),
        };
        self.variables
            .insert(name.clone(), VarValue::new(val_ptr, expr_type));
        Ok(())
    }

    pub fn build_set_var(&mut self, name: &Ident, value: &CrabValueType) -> Result<()> {
        trace!("Assigning to a variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        return match val_ptr_result {
            None => Err(CompileError::VarDoesNotExist(name.clone())),
            Some(var_value) => {
                if var_value.crab_type == value.get_crab_type() {
                    self.builder.build_store(
                        var_value.pointer,
                        value
                            .get_as_basic_value()
                            .ok_or(CompileError::InvalidNoneOption(String::from(
                                "build_set_var",
                            )))?,
                    );
                    Ok(())
                } else {
                    Err(CompileError::VarType(
                        name.clone(),
                        var_value.crab_type,
                        value.get_crab_type(),
                    ))
                }
            }
        };
    }

    pub fn build_retrieve_var(&mut self, name: &Ident) -> Result<CrabValueType<'ctx>> {
        trace!("Retreiving a variable with name {}", name);
        match self.variables.get(name) {
            Some(var_val) => match var_val.crab_type {
                CrabType::UINT => Ok(CrabValueType::new_uint(
                    self.builder
                        .build_load(var_val.pointer, name)
                        .into_int_value(),
                )),
                CrabType::STRING => Ok(CrabValueType::new_string(
                    self.builder
                        .build_load(var_val.pointer, name)
                        .into_pointer_value(),
                )),
                _ => unimplemented!(),
            },
            None => Err(CompileError::VarDoesNotExist(name.clone())),
        }
    }
}
