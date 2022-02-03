use crate::compile::llvmgen::Functiongen;
use crate::compile::Result;
use crate::parse::{CrabType, FnParam};
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::support::LLVMString;
use log::trace;
use std::path::PathBuf;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("main");
        Self { context, module }
    }

    //TODO: The linkage, mason! What does it mean?
    pub fn add_function(
        &mut self,
        name: &str,
        return_type: CrabType,
        params: &[FnParam],
        variadic: bool,
        linkage: Option<Linkage>,
    ) -> Result<()> {
        trace!(
            "Registering new function with name {} and {} args",
            name,
            params.len()
        );
        let fn_type = return_type.as_fn_type(self.context, params, variadic)?;
        let _fn_value = self.module.add_function(name, fn_type, linkage);
        Ok(())
    }

    pub fn get_function(&mut self, name: &str, args: &[FnParam]) -> Result<Functiongen<'ctx>> {
        Functiongen::new(name, &self.context, &self.module, args)
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.module.print_to_file(path)
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }
}
