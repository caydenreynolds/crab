use inkwell::AddressSpace;
use std::collections::HashMap;
use crate::compile::{CompileError, Result};
use crate::parse::{CrabType, Ident, TypedIdent};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::CallSiteValue;
use log::trace;
use uuid::Uuid;
use crate::compile::llvmgen::crab_value_type::CrabValueType;

pub struct Functiongen<'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    variables: HashMap<Ident, CrabValueType<'ctx>>,
}

impl<'ctx> Functiongen<'ctx> {
    pub fn new(
        name: &str,
        context: &'ctx Context,
        module: &Module<'ctx>,
        args: &[TypedIdent],
    ) -> Result<Functiongen<'ctx>> {
        trace!("Creating new functiongen for a function with name {}", name);
        let fn_value_opt = module.get_function(name);
        match fn_value_opt {
            Some(fn_value) => {
                let basic_block = context.append_basic_block(fn_value, "entry");
                let builder = context.create_builder();
                builder.position_at_end(basic_block);
                let variables = HashMap::new();
                let mut s = Self {
                    builder,
                    context,
                    variables,
                };

                // Add variables
                // Let's just immediately store them, because that's an easy way to make the types work out
                let mut n = 0;
                for arg in args {
                    s.build_create_var(&arg.name, arg.crab_type)?;
                    s.build_set_var(&arg.name, &CrabValueType::from_basic_value_enum(fn_value.get_nth_param(n).ok_or(CompileError::Internal(format!("Failed to get function because the param count did not match the expected number of params. i = {0}, fn_name = {1}", n, name)))?, arg.crab_type))?;
                    n += 1;
                }

                Ok(s)
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
        trace!("Building constant u64 with value {0}", value);
        CrabValueType::new_uint(self.context.i64_type().const_int(value, false))
    }

    pub fn build_const_string(&self, value: &String) -> Result<CrabValueType<'ctx>> {
        trace!("Building constant string with value {0}", value.clone());
        let str_ptr = self
            .builder
            .build_global_string_ptr(&value, &Uuid::new_v4().to_string())
            .as_pointer_value();
        Ok(CrabValueType::new_string(str_ptr))
    }

    pub fn build_const_bool(&self, value: bool) -> CrabValueType<'ctx> {
        trace!("Building constant bool with value {0}", value);
        let val_num = match value {
            true => 1,
            false => 0,
        };
        CrabValueType::new_bool(self.context.custom_width_int_type(1).const_int(val_num, false))
    }

    pub fn build_fn_call(
        &mut self,
        fn_name: &Ident,
        args: &[CrabValueType<'ctx>],
        module: &Module<'ctx>,
    ) -> Result<CallSiteValue<'ctx>> {
        trace!("Building a call to function {}", fn_name);
        let fn_value_opt = module.get_function(fn_name);
        let mut llvm_args = vec![];
        for arg in args {
            llvm_args.push(arg.try_as_basic_metadata_value()?);
        }
        match fn_value_opt {
            Some(fn_value) => Ok(self.builder.build_call(fn_value, llvm_args.as_slice(), "call")),
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
                CrabType::STRING => self
                    .builder
                    .build_alloca(self.context.i8_type().ptr_type(AddressSpace::Generic), name),
                CrabType::BOOL => self.builder.build_alloca(self.context.custom_width_int_type(1), name),
                _ => unimplemented!(),
            },
            _ => return Err(CompileError::VarAlreadyExists(name.clone())),
        };
        self.variables
            .insert(name.clone(), CrabValueType::new_ptr(val_ptr, expr_type));
        Ok(())
    }

    pub fn build_set_var(&mut self, name: &Ident, value: &CrabValueType) -> Result<()> {
        trace!("Assigning to a variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        return match val_ptr_result {
            None => Err(CompileError::VarDoesNotExist(name.clone())),
            Some(var_value) => {
                if var_value.get_crab_type() == value.get_crab_type() {
                    self.builder.build_store(
                        var_value.try_as_ptr_value()?,
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
                        var_value.get_crab_type(),
                        value.get_crab_type(),
                    ))
                }
            }
        };
    }

    pub fn build_retrieve_var(&mut self, name: &Ident) -> Result<CrabValueType<'ctx>> {
        trace!("Retreiving a variable with name {}", name);
        match self.variables.get(name) {
            Some(var_val) => match var_val.get_crab_type() {
                CrabType::UINT => Ok(CrabValueType::new_uint(
                    self.builder
                        .build_load(var_val.try_as_ptr_value()?, name)
                        .into_int_value(),
                )),
                CrabType::STRING => {
                    Ok(CrabValueType::new_string(
                    self.builder
                        .build_load(var_val.try_as_ptr_value()?, name)
                        .into_pointer_value(),
                ))},
                _ => unimplemented!(),
            },
            None => Err(CompileError::VarDoesNotExist(name.clone())),
        }
    }
}
