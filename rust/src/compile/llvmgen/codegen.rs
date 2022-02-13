use crate::compile::llvmgen::{add_builtins, add_main_func, FnManager, Functiongen, StructManager};
use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabAst, Func, FuncSignature, Struct};
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::support::LLVMString;
use log::trace;
use std::path::PathBuf;

pub struct Codegen<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    structs: StructManager,
    fns: FnManager,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn new(context: &'ctx Context, module: &'a Module<'ctx>) -> Self {
        let structs = StructManager::new();
        let fns = FnManager::new();
        let mut new = Self {
            context,
            module,
            structs,
            fns,
        };
        // Just unwrap this, because it should be impossible to fail
        add_builtins(&mut new).unwrap();
        new
    }

    pub fn compile(&mut self, ast: CrabAst) -> Result<()> {
        for crab_struct in &ast.structs {
            self.register_struct(crab_struct.clone())?;
        }
        for crab_struct in &ast.structs {
            self.build_struct_definition(crab_struct)?;
        }
        for func in &ast.functions {
            self.register_function(func.signature.clone(), false, None)?;
        }
        for func in &ast.functions {
            self.build_func(func)?;
        }
        add_main_func(self)?; // Really shouldn't fail either
        Ok(())
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.module.print_to_file(path)
    }

    // //TODO: The linkage, mason! What does it mean?
    pub fn register_function(
        &mut self,
        func: FuncSignature,
        variadic: bool,
        linkage: Option<Linkage>,
    ) -> Result<()> {
        let params = func.get_params();
        trace!(
            "Registering new function with name {} and {} args",
            func.name,
            params.len()
        );
        let fn_type =
            func.return_type
                .try_as_fn_type(self.context, self.module, &params, variadic)?;
        let _fn_value = self.module.add_function(&func.name, fn_type, linkage);
        self.fns.insert(func.name.clone(), func)?;
        Ok(())
    }

    pub fn get_context(&self) -> &Context {
        self.context
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        self.module
    }

    fn build_func(&mut self, func: &Func) -> Result<()> {
        let name = &func.signature.name;
        let params = func.signature.get_params();
        let mut fg = Functiongen::new(
            name,
            self.context,
            self.module,
            self.fns.clone(),
            self.structs.clone(),
            &params,
        )?;
        fg.build_codeblock(&func.body)?;
        return if fg.returns() {
            Ok(())
        } else {
            Err(CompileError::NoReturn(func.signature.name.clone()))
        };
    }

    fn register_struct(&mut self, strct: Struct) -> Result<()> {
        trace!("Building struct definition for struct {}", strct.name);
        self.structs.insert(strct.name.clone(), strct.clone())?;
        self.context.opaque_struct_type(&strct.name);
        Ok(())
    }

    fn build_struct_definition(&mut self, strct: &Struct) -> Result<()> {
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
