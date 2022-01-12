use crate::compile::llvmgen::Functiongen;
use crate::compile::Result;
use inkwell::context::Context;
use inkwell::module::Module;
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

    pub fn add_function(&mut self, name: &str) {
        trace!("Registering new function with name {}", name);
        let fn_type = self.context.i64_type().fn_type(&[], false);
        let fn_value = self.module.add_function(name, fn_type, None);
        self.context.append_basic_block(fn_value, "entry");
    }

    pub fn get_function(&mut self, name: &str) -> Result<Functiongen<'ctx>> {
        Functiongen::new(name, &self.context, &self.module)
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.module.print_to_file(path)
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
}
