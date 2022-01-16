use std::collections::HashMap;
// use inkwell::basic_block::BasicBlock;
use crate::compile::llvmgen::VarValue;
use crate::compile::BasicValueType;
use crate::compile::{CompileError, Result};
use crate::parse::{CrabType, Ident};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{ArrayValue, CallSiteValue, IntValue, VectorValue};
use log::trace;

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
    pub fn build_return(&mut self, value: &BasicValueType<'ctx>) {
        trace!("Building return statement");
        match value {
            BasicValueType::IntValue(value, _) => self.builder.build_return(Some(value)),
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

    pub fn build_const_string(&self, value: &String) -> BasicValueType<'ctx> {
        trace!("Building constant string");
        BasicValueType::StringPrimitiveValue(
            self.context.const_string(value.as_bytes(), true),
            CrabType::STRING((value.len() + 1) as u64),
        )
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
                CrabType::STRING(size) => self.builder.build_array_alloca(
                    self.context.i8_type().array_type(size as u32),
                    self.context.i64_type().const_int(size, false),
                    name,
                ),
                _ => unimplemented!(),
            },
            _ => return Err(CompileError::VarAlreadyExists(name.clone())),
        };
        self.variables
            .insert(name.clone(), VarValue::new(val_ptr, expr_type));
        Ok(())
    }

    pub fn build_set_var(&mut self, name: &Ident, value: &BasicValueType) -> Result<()> {
        trace!("Assigning to a variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        // Only support i64 constants for now, and error on attempting to mutate a variable
        match val_ptr_result {
            None => Err(CompileError::VarDoesNotExist(name.clone())),
            Some(var_value) => {
                match value {
                    BasicValueType::IntValue(v, ct) => {
                        if *ct == var_value.crab_type {
                            self.builder.build_store(var_value.pointer, *v);
                        } else {
                            return Err(CompileError::VarType(
                                name.clone(),
                                var_value.crab_type,
                                *ct,
                            ));
                        }
                    }
                    BasicValueType::CallValue(v, ct) => {
                        if *ct == var_value.crab_type {
                            self.builder.build_store(
                                var_value.pointer,
                                v.try_as_basic_value()
                                    .expect_left("Expected function to return a basic value"),
                            );
                        } else {
                            return Err(CompileError::VarType(
                                name.clone(),
                                var_value.crab_type,
                                *ct,
                            ));
                        }
                    }
                    BasicValueType::StringPrimitiveValue(v, ct) => {
                        if *ct == var_value.crab_type {
                            self.builder.build_store(var_value.pointer, *v);
                        } else {
                            return Err(CompileError::VarType(
                                name.clone(),
                                var_value.crab_type,
                                *ct,
                            ));
                        }
                    }
                    BasicValueType::StringValue(v, ct) => {
                        if *ct == var_value.crab_type {
                            self.builder.build_store(var_value.pointer, *v);
                        } else {
                            return Err(CompileError::VarType(
                                name.clone(),
                                var_value.crab_type,
                                *ct,
                            ));
                        }
                    }
                    _ => unimplemented!(),
                };
                Ok(())
            }
        }
    }

    pub fn build_retrieve_var(&mut self, name: &Ident) -> Result<BasicValueType<'ctx>> {
        trace!("Retreiving a variable with name {}", name);
        match self.variables.get(name) {
            Some(var_val) => match var_val.crab_type {
                CrabType::UINT => Ok(BasicValueType::IntValue(
                    self.builder
                        .build_load(var_val.pointer, name)
                        .into_int_value(),
                    var_val.crab_type,
                )),
                CrabType::STRING(_) => Ok(BasicValueType::StringValue(
                    self.builder
                        .build_load(var_val.pointer, name)
                        .into_array_value(),
                    var_val.crab_type,
                )),
                _ => unimplemented!(),
            },
            None => Err(CompileError::VarDoesNotExist(name.clone())),
        }
    }
}
