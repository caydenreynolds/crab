use crate::compile::{
    add_builtins, add_main_func, CompileError, FnManager, Functiongen, Result, TypeManager,
};
use crate::parse::ast::{CrabAst, CrabInterface, Func, Struct};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::support::LLVMString;
use log::trace;
use std::path::PathBuf;

pub struct Codegen<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    types: TypeManager,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        let structs = TypeManager::new();
        Self {
            context,
            module,
            types: structs,
        }
    }

    pub fn compile(&mut self, ast: CrabAst) -> Result<()> {
        for crab_struct in &ast.structs {
            self.register_struct(crab_struct.clone())?;
        }
        for crab_struct in &ast.structs {
            self.build_struct_definition(crab_struct)?;
        }

        for interface in &ast.interfaces {
            self.register_interface(interface.1.clone())?;
        }
        for intr in &ast.intrs {
            self.types.register_intr(intr.clone())?;
        }

        let mut fns = FnManager::new(self.types.clone(), self.context, self.module);
        add_builtins(self, &mut fns)?;

        for func in &ast.functions {
            fns.add_source(func.clone());
        }
        fns.add_main_to_queue()?;
        while let Some(func) = fns.pop_build_queue() {
            // self.register_function(func.signature.clone(), false, None)?;
            self.build_func(func, &mut fns)?;
        }

        add_main_func(self, &mut fns)?; // Really shouldn't fail either
        Ok(())
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.module.print_to_file(path)
    }

    pub fn get_context(&self) -> &Context {
        self.context
    }
    pub fn get_module(&self) -> &Module<'ctx> {
        self.module
    }

    fn build_func(&mut self, func: Func, fns: &mut FnManager<'a, 'ctx>) -> Result<()> {
        let name = func.signature.name.clone();
        let params = func.signature.get_params();
        let mut fg = Functiongen::new(
            &name,
            self.context,
            self.module,
            fns,
            self.types.clone(),
            &params,
        )?;
        fg.build_codeblock(&func.body)?;
        return if fg.returns() {
            Ok(())
        } else {
            Err(CompileError::NoReturn(func.signature.name.clone()))
        };
    }

    fn register_interface(&mut self, intfc: CrabInterface) -> Result<()> {
        trace!("Registering interface {}", intfc.name);
        self.types.register_interface(intfc)?;
        Ok(())
    }

    fn register_struct(&mut self, strct: Struct) -> Result<()> {
        trace!("Registering struct {}", strct.name);
        self.types.register_struct(strct.clone())?;
        self.context.opaque_struct_type(&strct.name);
        Ok(())
    }

    fn build_struct_definition(&mut self, strct: &Struct) -> Result<()> {
        trace!("Building struct definition for struct {}", strct.name);
        let st = self
            .module
            .get_struct_type(&strct.name)
            .ok_or(CompileError::StructDoesNotExist(strct.name.clone()))?;
        st.set_body(
            &strct.get_fields_as_basic_type(self.context, self.module)?,
            false,
        );
        Ok(())
    }
}
