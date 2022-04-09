use crate::compile::crab_value_type::CrabValueType;
use crate::compile::CompileError::MallocErr;
use crate::compile::{
    add_builtin_definition, CompileError, FnManager, ManagedType, Result, TypeManager, VarManager,
};
use crate::parse::ast::{
    Assignment, BodyType, CodeBlock, CrabType, DoWhileStmt, ElseStmt, Expression, ExpressionType,
    FnCall, FnParam, IfStmt, Primitive, Statement, StructInit, WhileStmt,
};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use log::trace;
use std::collections::HashMap;

pub struct Functiongen<'a, 'b, 'ctx> {
    pub name: String,
    pub builder: Builder<'ctx>,
    pub context: &'ctx Context,
    pub module: &'a Module<'ctx>,
    pub fns: &'b mut FnManager<'a, 'ctx>,
    pub structs: TypeManager,
    pub variables: VarManager<'ctx>,
    pub fn_value: FunctionValue<'ctx>,
}

impl<'a, 'b, 'ctx> Functiongen<'a, 'b, 'ctx> {
    pub fn new(
        name: &str,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        fns: &'b mut FnManager<'a, 'ctx>,
        structs: TypeManager,
        args: &[FnParam],
    ) -> Result<Functiongen<'a, 'b, 'ctx>> {
        trace!("Creating new functiongen for a function with name {}", name);
        let fn_value_opt = module.get_function(name);
        match fn_value_opt {
            Some(fn_value) => {
                let basic_block = context.append_basic_block(fn_value, "entry");
                let builder = context.create_builder();
                builder.position_at_end(basic_block);
                let variables = VarManager::new();
                let mut s = Self {
                    builder,
                    context,
                    module,
                    variables,
                    fns,
                    structs,
                    fn_value,
                    name: String::from(name),
                };

                let mut n = 0;
                for arg in args {
                    s.variables.assign(arg.name.clone(), CrabValueType::from_basic_value_enum(fn_value.get_nth_param(n).ok_or(CompileError::Internal(format!("Failed to get function because the param count did not match the expected number of params. i = {0}, fn_name = {1}", n, name)))?, arg.crab_type.clone()))?;
                    n += 1;
                }

                Ok(s)
            }
            // This should never happen
            None => Err(CompileError::CouldNotFindFunction(String::from(name))),
        }
    }

    ///
    /// Build llvm ir for this function
    /// If the bt is a codeblock, that codeblock will be built
    /// If the bt is a compiler builtin, builtins will be used to generate the implementation
    ///
    /// Params:
    /// - `bt`: The function's body type
    ///
    /// Returns:
    /// True if
    pub fn build(&mut self, bt: BodyType) -> Result<()> {
        match bt {
            BodyType::CODEBLOCK(cb) => {
                let returns = self.build_codeblock(&cb)?;
                if returns {
                    Ok(())
                } else {
                    Err(CompileError::NoReturn(self.name.clone()))
                }
            }
            BodyType::COMPILER_PROVIDED => add_builtin_definition(self),
        }
    }

    ///
    /// Build llvm ir for a given codeblock
    ///
    /// Params:
    /// - `cb`: The codeblock to build
    ///
    /// Returns:
    /// True if the built codeblock will always return a value, or false otherwise
    ///
    fn build_codeblock(&mut self, cb: &CodeBlock) -> Result<bool> {
        let returns = cb.statements.iter().try_fold(false, |returns, stmt| {
            if !returns {
                self.build_statement(stmt)
            } else {
                Ok(true)
            }
        })?;
        Ok(returns)
    }

    ///
    /// Builds llvm ir for a given statement
    ///
    /// Params:
    /// - `stmt`: The statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_statement(&mut self, stmt: &Statement) -> Result<bool> {
        return match &stmt {
            Statement::IF_STATEMENT(is) => self.build_if_stmt(&is),
            Statement::WHILE_STATEMENT(ws) => self.build_while_stmt(&ws),
            Statement::DO_WHILE_STATEMENT(dws) => self.build_do_while_stmt(&dws),
            Statement::EXPRESSION(expr) => {
                self.build_expression(expr, None)?;
                Ok(false)
            }
            Statement::ASSIGNMENT(ass) => self.build_assignment(ass),
            Statement::REASSIGNMENT(reass) => self.build_reassignment(reass),
            Statement::RETURN(ret) => self.build_return(ret),
        };
    }

    ///
    /// Builds llvm ir for a given assignment statement
    /// This function always returns false
    ///
    /// Params:
    /// - `ass`: The assignment to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_assignment(&mut self, ass: &Assignment) -> Result<bool> {
        trace!("Assigning to a variable with name {:?}", ass.var_name);
        let bv = self.build_expression(&ass.expr, None)?;
        self.variables.assign(ass.var_name.clone(), bv)?;
        Ok(false)
    }

    ///
    /// Builds llvm ir for a given reassignment statement
    /// This function always returns false
    ///
    /// Params:
    /// - `ass`: The assignment to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_reassignment(&mut self, ass: &Assignment) -> Result<bool> {
        trace!("Reassigning to a variable with name {:?}", ass.var_name);
        let bv = self.build_expression(&ass.expr, None)?;
        if let CrabType::STRUCT(_) = bv.get_crab_type() {
            let src = self
                .builder
                .build_load(bv.try_as_struct_value()?, "reassignment");
            let dest = self.variables.get(&ass.var_name)?.try_as_struct_value()?;
            self.builder.build_store(dest, src);
        } else {
            self.variables.reassign(ass.var_name.clone(), bv)?;
        }
        Ok(false)
    }

    ///
    /// Builds llvm ir for a given if statement
    ///
    /// Params:
    /// - `is`: The if statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_if_stmt(&mut self, is: &IfStmt) -> Result<bool> {
        // Setup the stuff we need
        let bv = self.build_expression(&is.expr, None)?;
        let then_block = self.context.append_basic_block(self.fn_value, "if");
        let else_block = self.context.append_basic_block(self.fn_value, "else");
        let always_block = self.context.append_basic_block(self.fn_value, "always");

        // The if
        self.builder
            .build_conditional_branch(bv.try_as_bool_value()?, then_block, else_block);
        self.builder.position_at_end(then_block);
        let if_returns = self.build_codeblock(&is.then)?;
        if !if_returns {
            self.builder.build_unconditional_branch(always_block);
        }

        // The else
        self.builder.position_at_end(else_block);
        let else_returns = if let Some(es) = &is.else_stmt {
            match es.as_ref() {
                ElseStmt::ELSE(cb) => self.build_codeblock(&cb),
                ElseStmt::ELIF(ifs) => self.build_if_stmt(&ifs),
            }?
        } else {
            false
        };
        if !else_returns {
            self.builder.build_unconditional_branch(always_block);
        }

        // The ugly
        self.builder.position_at_end(always_block);
        let always_returns = if_returns && else_returns;
        if always_returns {
            self.build_unreachable()?;
        }
        Ok(always_returns)
    }

    fn build_expression(
        &mut self,
        expr: &Expression,
        previous: Option<CrabValueType<'ctx>>,
    ) -> Result<CrabValueType<'ctx>> {
        let val = match &expr.this {
            ExpressionType::STRUCT_INIT(si) => self.build_struct_init(si),
            ExpressionType::PRIM(prim) => self.build_primitive(prim),
            ExpressionType::FN_CALL(fc) => match previous {
                None => self.build_fn_call(fc, None),
                Some(prev) => self.build_fn_call(fc, Some(prev)),
            },
            ExpressionType::VARIABLE(id) => match previous {
                None => self.variables.get(id),
                Some(prev) => {
                    let name = prev.try_get_struct_name()?;
                    let cs = match self.structs.get_type(&name)? {
                        ManagedType::STRUCT(strct) => strct.clone(),
                        ManagedType::INTERFACE(_) => return Err(CompileError::NotAStruct),
                    };
                    let field_index = cs.get_field_index(id)?;
                    let source_ptr = self
                        .builder
                        .build_struct_gep(prev.try_as_struct_value()?, field_index as u32, "source")
                        .or(Err(CompileError::Gep(String::from(
                            "functiongen::build_expression_chain",
                        ))))?;
                    let val = self.builder.build_load(source_ptr, "dest");
                    Ok(CrabValueType::from_basic_value_enum(
                        val,
                        cs.get_field_crab_type(id)?,
                    ))
                }
            },
        }?;

        match &expr.next {
            None => Ok(val),
            Some(next) => self.build_expression(next, Some(val)),
        }
    }

    fn build_struct_init(&mut self, si: &StructInit) -> Result<CrabValueType<'ctx>> {
        let crab_struct = match self.structs.get_type(&si.name)? {
            ManagedType::STRUCT(strct) => strct.clone(),
            ManagedType::INTERFACE(_) => return Err(CompileError::NotAStruct),
        };
        let st = self
            .module
            .get_struct_type(&crab_struct.name)
            .ok_or(CompileError::StructDoesNotExist(si.name.clone()))?;

        if crab_struct.fields.len() != si.fields.len() {
            return Err(CompileError::StructInitFieldCount(
                si.name.clone(),
                crab_struct.fields.len(),
                si.fields.len(),
            ));
        }

        let mut field_vals = HashMap::new();
        for field in &si.fields {
            let val = self.build_expression(&field.value, None)?;
            field_vals.insert(field.name.clone(), val);
        }

        let mut init_field_list = vec![];
        for field in &crab_struct.fields {
            let val = field_vals
                .get(&field.name)
                .ok_or(CompileError::StructInitFieldName(
                    si.name.clone(),
                    field.name.clone(),
                ))?;
            init_field_list.push(val.get_as_basic_value().ok_or(
                CompileError::InvalidNoneOption(String::from("build_struct_init")),
            )?);
        }

        //TODO: free
        let new_struct_ptr = self
            .builder
            .build_malloc(st, "struct_init")
            .or(Err(MallocErr(String::from("Funcgen::build_struct_init"))))?;

        for i in 0..init_field_list.len() {
            let init_field = init_field_list.get(i).unwrap();

            let element_ptr = self
                .builder
                .build_struct_gep(new_struct_ptr, i as u32, "element_ptr")
                .or(Err(CompileError::Gep(String::from(
                    "functiongen::build_struct_init",
                ))))?;
            self.builder.build_store(element_ptr, *init_field);
        }

        Ok(CrabValueType::new_struct(new_struct_ptr, si.name.clone()))
    }

    fn build_primitive(&mut self, prim: &Primitive) -> Result<CrabValueType<'ctx>> {
        return match prim {
            Primitive::STRING(str) => self.build_const_string(str),
            Primitive::BOOL(bl) => Ok(self.build_const_bool(*bl)),
            Primitive::UINT(uint) => Ok(self.build_const_u64(*uint)),
        };
    }

    fn build_fn_call(
        &mut self,
        call: &FnCall,
        caller_opt: Option<CrabValueType<'ctx>>,
    ) -> Result<CrabValueType<'ctx>> {
        trace!("Building a call to function {:#?}", call.name);

        // Handle all of the positional arguments
        let mut unnamed_args = vec![];
        // Add 'self' if this is a method call
        if let Some(caller) = caller_opt.clone() {
            unnamed_args.push(caller);
        }
        for arg in &call.unnamed_args {
            unnamed_args.push(self.build_expression(arg, None)?);
        }

        // Handle all of the optional arguments
        let mut named_args = HashMap::new();
        for named_arg in &call.named_args {
            named_args.insert(
                named_arg.name.clone(),
                self.build_expression(&named_arg.expr, None)?,
            );
        }

        let fn_header = self
            .fns
            .get_signature(call.clone(), caller_opt, &unnamed_args, &named_args)?
            .clone();

        // Build the args array
        let mut args = vec![];
        for arg in unnamed_args {
            args.push(arg.try_as_basic_metadata_value()?);
        }
        for named_param in fn_header.named_params {
            match named_args.get(&named_param.name) {
                Some(arg) => args.push(arg.try_as_basic_metadata_value()?),
                None => args.push(
                    self.build_expression(&named_param.expr, None)?
                        .try_as_basic_metadata_value()?,
                ),
            }
        }

        // Build the IR
        let fn_value = self
            .module
            .get_function(&fn_header.name)
            .ok_or(CompileError::CouldNotFindFunction(call.name.clone()))?;
        let csv = self.builder.build_call(fn_value, &args, "call");

        Ok(CrabValueType::from_call_site_value(
            csv,
            fn_header.return_type.clone(),
        ))
    }

    ///
    /// Builds llvm ir for a given return statement
    /// This function always returns true
    ///
    /// Params:
    /// - `expr`: The optional expression to return
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    pub fn build_return(&mut self, expr: &Option<Expression>) -> Result<bool> {
        trace!("Building return statement");
        match expr {
            None => self.builder.build_return(None),
            Some(expr) => {
                let bv = self.build_expression(&expr, None)?;
                match bv.get_as_basic_value() {
                    Some(x) => self.builder.build_return(Some(&x)),
                    None => unreachable!(),
                }
            }
        };
        Ok(true)
    }

    fn build_const_u64(&self, value: u64) -> CrabValueType<'ctx> {
        trace!("Building constant u64 with value {0}", value);
        CrabValueType::new_uint(self.context.i64_type().const_int(value, false))
    }

    fn build_const_string(&self, value: &String) -> Result<CrabValueType<'ctx>> {
        trace!("Building constant string with value {0}", value.clone());
        let str_ptr = self
            .builder
            .build_global_string_ptr(&value, "global_str")
            .as_pointer_value();
        Ok(CrabValueType::new_string(str_ptr))
    }

    fn build_const_bool(&self, value: bool) -> CrabValueType<'ctx> {
        trace!("Building constant bool with value {0}", value);
        let val_num = match value {
            true => 1,
            false => 0,
        };
        CrabValueType::new_bool(
            self.context
                .custom_width_int_type(1)
                .const_int(val_num, false),
        )
    }

    pub fn build_unreachable(&mut self) -> Result<()> {
        self.builder.build_unreachable();
        Ok(())
    }

    ///
    /// Builds llvm ir for a given while statement
    ///
    /// Params:
    /// - `ws`: The while statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_while_stmt(&mut self, ws: &WhileStmt) -> Result<bool> {
        let bv = self.build_expression(&ws.expr, None)?;
        let then_block = self.context.append_basic_block(self.fn_value, "while");
        let always_block = self.context.append_basic_block(self.fn_value, "always");

        self.builder
            .build_conditional_branch(bv.try_as_bool_value()?, then_block, always_block);

        self.build_loop(&ws.then, then_block, always_block, &ws.expr)
    }

    ///
    /// Builds llvm ir for a given do-while statement
    ///
    /// Params:
    /// - `dws`: The do-while statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_do_while_stmt(&mut self, dws: &DoWhileStmt) -> Result<bool> {
        let then_block = self.context.append_basic_block(self.fn_value, "do-while");
        let always_block = self.context.append_basic_block(self.fn_value, "always");

        self.builder.build_unconditional_branch(then_block);

        self.build_loop(&dws.then, then_block, always_block, &dws.expr)
    }

    ///
    /// Builds the guts of a loop
    /// Includes the loop body and the conditional branch
    /// Resets the builder position at the end of the loop
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_loop(
        &mut self,
        then: &CodeBlock,
        then_block: BasicBlock,
        always_block: BasicBlock,
        condition: &Expression,
    ) -> Result<bool> {
        self.builder.position_at_end(then_block);
        let returns = self.build_codeblock(then)?;
        // Rebuild the condition
        let bv = self.build_expression(condition, None)?;
        self.builder
            .build_conditional_branch(bv.try_as_bool_value()?, then_block, always_block);

        // The always block will be built by the caller
        // I don't really like having this kind of side effect
        // How can we transfer that responsibility into this function?
        self.builder.position_at_end(always_block);

        Ok(returns)
    }
}
