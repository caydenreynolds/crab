use crate::compile::llvmgen::Functiongen;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::LLVMString;
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

    pub fn add_function(&mut self, name: &str) -> Functiongen<'ctx> {
        Functiongen::new(name, &self.context, &self.module)
    }

    pub fn print_to_file(&self, path: PathBuf) -> Result<(), LLVMString> {
        self.module.print_to_file(path)
    }
}
