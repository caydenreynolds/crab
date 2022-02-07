use crate::compile::llvmgen::{FnManager, Functiongen, StructManager};
use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabAst, CrabType, FnParam, Func, FuncSignature, Ident, Struct};
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use log::trace;
use std::path::PathBuf;
use inkwell::support::LLVMString;

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
        let mut new = Self { context, module, structs, fns };
        // Just unwrap this, because it should be impossible to fail
        // Yes I know this is bad practice, but it's been a long day and this is all temporary code anyway
        new.add_builtin_fns().unwrap();
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
        Ok(())
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.module.print_to_file(path)
    }

    fn add_builtin_fns(&mut self) -> Result<()> {
        //TODO: If a use writes a function with the same name as an internal builtin (e.g. prinf), it overwrites the real printf and causes llc to barf
        //TODO: Writing a function named __printf__ also causes an overwrite (I think) and this would probably be ok, except we need to make sure it's a local overwrite, not a global one. Namespacing should fix this issue.
        //TODO: Name mangling would fix this
        self.add_printf()
    }

    /// define and add the printf function to the module
    fn add_printf(&mut self) -> Result<()> {
        let signature = FuncSignature {
            name: Ident::from("printf"),
            return_type: CrabType::FLOAT,
            unnamed_params: vec![FnParam {
                name: Ident::from("str"),
                crab_type: CrabType::STRING,
            }],
            named_params: vec![],
        };
        self.register_function(signature.clone(), true, Some(Linkage::External))?;
        self.fns.insert(Ident::from("__printf__"), signature)?;
        Ok(())
    }

    // //TODO: The linkage, mason! What does it mean?
    fn register_function(
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
        let fn_type = func.return_type.try_as_fn_type(self.context, self.module, &params, variadic)?;
        let _fn_value = self.module.add_function(&func.name, fn_type, linkage);
        self.fns.insert(func.name.clone(), func)?;
        Ok(())
    }

    fn build_func(&mut self, func: &Func) -> Result<()> {
        let name = &func.signature.name;
        let params = func.signature.get_params();
        let mut fg = Functiongen::new(name, self.context, self.module, self.fns.clone(), self.structs.clone(), &params)?;
        fg.build_codeblock(&func.body)?;
        return if fg.returns() {
            Ok(())
        } else {
            Err(CompileError::NoReturn(func.signature.name.clone()))
        }
    }

    fn register_struct(&mut self, strct: Struct) -> Result<()> {
        trace!("Building struct definition for struct {}", strct.name);
        self.structs.insert(strct.name.clone(), strct.clone())?;
        self.context.opaque_struct_type(&strct.name);
        Ok(())
    }

    fn build_struct_definition(&mut self, strct: &Struct) -> Result<()> {
        let st = self.module.get_struct_type(&strct.name).ok_or(CompileError::StructDoesNotExist(strct.name.clone()))?;
        st.set_body(&strct.get_fields_as_basic_type(self.context, self.module)?, true);
        Ok(())
    }
}
