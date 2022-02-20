use crate::compile::llvmgen::crab_value_type::CrabValueType;
use crate::compile::llvmgen::{FnManager, StructManager, VarManager};
use crate::compile::{CompileError, Result};
use crate::parse::ast::{
    Assignment, CodeBlock, CrabType, DoWhileStmt, ElseStmt, Expression, ExpressionChain,
    ExpressionChainType, FnCall, FnParam, IfStmt, Primitive, Statement, StructInit, WhileStmt,
};
use crate::parse::mangle_function_name;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FunctionValue;
use log::trace;
use std::collections::HashMap;

pub struct Functiongen<'a, 'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    fns: FnManager,
    structs: StructManager,
    variables: VarManager<'ctx>,
    else_stack: Vec<BasicBlock<'ctx>>,
    always_stack: Vec<BasicBlock<'ctx>>,
    fn_value: FunctionValue<'ctx>,
    current_basic_block: BasicBlock<'ctx>,
    codeblock_returns: bool,
}

impl<'a, 'ctx> Functiongen<'a, 'ctx> {
    pub fn new(
        name: &str,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        fns: FnManager,
        structs: StructManager,
        args: &[FnParam],
    ) -> Result<Functiongen<'a, 'ctx>> {
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
                    else_stack: vec![],
                    always_stack: vec![],
                    fn_value,
                    current_basic_block: basic_block,
                    codeblock_returns: false,
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

    pub fn returns(&self) -> bool {
        self.codeblock_returns
    }

    pub fn build_codeblock(&mut self, cb: &CodeBlock) -> Result<()> {
        self.codeblock_returns = false;
        for stmt in &cb.statements {
            self.build_statement(stmt)?;

            if self.codeblock_returns {
                break;
            }
        }
        Ok(())
    }

    fn build_statement(&mut self, stmt: &Statement) -> Result<()> {
        match &stmt {
            Statement::IF_STATEMENT(is) => self.build_if_stmt(&is)?,
            Statement::WHILE_STATEMENT(ws) => self.build_while_stmt(&ws)?,
            Statement::DO_WHILE_STATEMENT(dws) => self.build_do_while_stmt(&dws)?,
            Statement::EXPRESSION_CHAIN(ec) => {
                self.build_expression_chain(ec, None)?;
            }
            Statement::ASSIGNMENT(ass) => self.build_assignment(ass)?,
            Statement::REASSIGNMENT(reass) => self.build_reassignment(reass)?,
            Statement::RETURN(ret) => self.build_return(ret)?,
        };
        Ok(())
    }

    fn build_assignment(&mut self, ass: &Assignment) -> Result<()> {
        trace!("Assigning to a variable with name {:?}", ass.var_name);
        let bv = self.build_expression(&ass.expr)?;
        self.variables.assign(ass.var_name.clone(), bv)?;
        Ok(())
    }

    fn build_reassignment(&mut self, ass: &Assignment) -> Result<()> {
        trace!("Reassigning to a variable with name {:?}", ass.var_name);
        let bv = self.build_expression(&ass.expr)?;
        if let CrabType::STRUCT(_) = bv.get_crab_type() {
            let src = self
                .builder
                .build_load(bv.try_as_struct_value()?, "reassignment");
            let dest = self.variables.get(&ass.var_name)?.try_as_struct_value()?;
            self.builder.build_store(dest, src);
        } else {
            self.variables.reassign(ass.var_name.clone(), bv)?;
        }
        Ok(())
    }

    fn build_while_stmt(&mut self, ws: &WhileStmt) -> Result<()> {
        let bv = self.build_expression(&ws.expr)?;
        self.begin_while(&bv)?;
        self.build_codeblock(&ws.then)?;
        let bv = self.build_expression(&ws.expr)?;
        self.end_while(&bv)?;
        Ok(())
    }

    fn build_do_while_stmt(&mut self, dws: &DoWhileStmt) -> Result<()> {
        self.begin_do_while()?;
        self.build_codeblock(&dws.then)?;
        let bv = self.build_expression(&dws.expr)?;
        self.end_while(&bv)?;
        Ok(())
    }

    fn build_if_stmt(&mut self, is: &IfStmt) -> Result<()> {
        let bv = self.build_expression(&is.expr)?;
        self.begin_if_then(&bv)?;
        self.build_codeblock(&is.then)?;
        let mut returns = self.codeblock_returns;
        self.begin_if_else()?; // Always begin if else -- even if no else block present, we still need a jump instruction
        if let Some(es) = &is.else_stmt {
            match es.as_ref() {
                ElseStmt::ELSE(cb) => self.build_codeblock(cb)?,
                ElseStmt::ELIF(ifs) => self.build_if_stmt(ifs)?,
            }
            returns = returns && self.codeblock_returns;
        } else {
            returns = false
        }
        self.end_if()?;
        self.codeblock_returns = returns;
        if returns {
            self.build_unreachable()?;
        }
        Ok(())
    }

    fn build_expression(&mut self, expr: &Expression) -> Result<CrabValueType<'ctx>> {
        return match expr {
            Expression::STRUCT_INIT(si) => self.build_struct_init(si),
            Expression::PRIM(prim) => self.build_primitive(prim),
            Expression::CHAIN(ec) => self.build_expression_chain(ec, None),
        };
    }

    fn build_expression_chain(
        &mut self,
        ec: &ExpressionChain,
        previous: Option<CrabValueType<'ctx>>,
    ) -> Result<CrabValueType<'ctx>> {
        trace!("Building expression chain {:?}", ec);
        let result = match previous {
            None => match &ec.this {
                ExpressionChainType::VARIABLE(id) => self.variables.get(id)?,
                ExpressionChainType::FN_CALL(fc) => self.build_fn_call(fc, None)?,
            },
            Some(prev) => match &ec.this {
                ExpressionChainType::VARIABLE(id) => {
                    let name = prev.try_get_struct_name()?;
                    let cs = self.structs.get(&name)?;
                    let field_index = cs.get_field_index(id)?;
                    let source_ptr = self
                        .builder
                        .build_struct_gep(prev.try_as_struct_value()?, field_index as u32, "source")
                        .or(Err(CompileError::Gep(String::from(
                            "functiongen::build_expression_chain",
                        ))))?;
                    let val = self.builder.build_load(source_ptr, "dest");
                    CrabValueType::from_basic_value_enum(val, cs.get_field_crab_type(id)?)
                }
                ExpressionChainType::FN_CALL(fc) => self.build_fn_call(fc, Some(prev))?,
            },
        };

        match &ec.next {
            None => Ok(result),
            Some(next) => self.build_expression_chain(next, Some(result)),
        }
    }

    fn build_struct_init(&mut self, si: &StructInit) -> Result<CrabValueType<'ctx>> {
        let crab_struct = self.structs.get(&si.name)?.clone();
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
            let val = self.build_expression(&field.value)?;
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

        let new_struct_ptr = self.builder.build_alloca(st, "struct_init");

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
        let mangled_name = mangle_function_name(
            &call.name,
            caller_opt
                .clone()
                .map(|ct| {
                    ct.try_get_struct_name()
                        .expect("Method called on a type that is not a struct")
                })
                .as_ref(),
        );
        let fn_header = self.fns.get(&mangled_name)?.clone();

        let supplied_pos_arg_count = match caller_opt {
            None => call.unnamed_args.len(),
            Some(_) => call.unnamed_args.len() + 1,
        };

        // Check to make sure we have exactly the arguments we expect
        if supplied_pos_arg_count != fn_header.unnamed_params.len() {
            return Err(CompileError::PositionalArgumentCount(
                fn_header.name.clone(),
                fn_header.unnamed_params.len(),
                call.unnamed_args.len(),
            ));
        }
        for named_expr in &call.named_args {
            if !fn_header
                .named_params
                .iter()
                .any(|param| param.name == named_expr.name)
            {
                return Err(CompileError::InvalidNamedArgument(
                    fn_header.name.clone(),
                    named_expr.name.clone(),
                ));
            }
        }

        let mut args = vec![];

        // Add 'self' if this is a method call
        if let Some(caller) = caller_opt {
            args.push(caller.try_as_basic_metadata_value()?);
        }

        // Handle all of the positional arguments
        for arg in &call.unnamed_args {
            args.push(self.build_expression(arg)?.try_as_basic_metadata_value()?);
        }

        // Handle all of the optional arguments
        for named_param in fn_header.named_params {
            let mut arg_found = false;
            for named_arg in &call.named_args {
                if named_param.name == named_arg.name {
                    arg_found = true;
                    args.push(
                        self.build_expression(&named_arg.expr)?
                            .try_as_basic_metadata_value()?,
                    );
                }
            }

            if !arg_found {
                args.push(
                    self.build_expression(&named_param.expr)?
                        .try_as_basic_metadata_value()?,
                );
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

    pub fn build_return(&mut self, expr: &Option<Expression>) -> Result<()> {
        trace!("Building return statement");
        self.codeblock_returns = true;
        match expr {
            None => self.builder.build_return(None),
            Some(expr) => {
                let bv = self.build_expression(&expr)?;
                match bv.get_as_basic_value() {
                    Some(x) => self.builder.build_return(Some(&x)),
                    None => unreachable!(),
                }
            }
        };
        Ok(())
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

    fn begin_if_then(&mut self, condition: &CrabValueType) -> Result<()> {
        let then_block = self.context.append_basic_block(self.fn_value, "if_then");
        let else_block = self.context.append_basic_block(self.fn_value, "else");
        let always_block = self.context.append_basic_block(self.fn_value, "always");

        self.builder.build_conditional_branch(
            condition.try_as_bool_value()?,
            then_block,
            else_block,
        );

        self.else_stack.push(else_block);
        self.always_stack.push(always_block);

        self.builder.position_at_end(then_block);
        self.current_basic_block = then_block;

        Ok(())
    }

    fn begin_if_else(&mut self) -> Result<()> {
        // Only build a branch to end instruction if this block does not return
        if !self.codeblock_returns {
            let always_block = self
                .always_stack
                .pop()
                .ok_or(CompileError::EmptyStack(String::from("Always stack")))?;
            self.add_terminating_instruction(self.current_basic_block, always_block);
            self.always_stack.push(always_block);
        }

        let else_block = self
            .else_stack
            .pop()
            .ok_or(CompileError::EmptyStack(String::from("else_stack")))?;
        self.builder.position_at_end(else_block);
        self.current_basic_block = else_block;
        Ok(())
    }

    fn end_if(&mut self) -> Result<()> {
        let always_block = self
            .always_stack
            .pop()
            .ok_or(CompileError::EmptyStack(String::from("always_stack")))?;

        if !self.codeblock_returns {
            self.add_terminating_instruction(self.current_basic_block, always_block);
        }

        self.builder.position_at_end(always_block);
        self.current_basic_block = always_block;
        Ok(())
    }

    fn begin_while(&mut self, condition: &CrabValueType) -> Result<()> {
        let then_block = self.context.append_basic_block(self.fn_value, "while");
        let always_block = self.context.append_basic_block(self.fn_value, "always");
        self.always_stack.push(always_block);
        self.builder.build_conditional_branch(
            condition.try_as_bool_value()?,
            then_block,
            always_block,
        );
        self.builder.position_at_end(then_block);
        self.current_basic_block = then_block;
        Ok(())
    }

    fn begin_do_while(&mut self) -> Result<()> {
        let then_block = self.context.append_basic_block(self.fn_value, "do_while");
        let always_block = self.context.append_basic_block(self.fn_value, "always");
        self.always_stack.push(always_block);
        self.builder.build_unconditional_branch(then_block);
        self.builder.position_at_end(then_block);
        self.current_basic_block = then_block;
        Ok(())
    }

    /// Used to terminate a while or do while block
    fn end_while(&mut self, condition: &CrabValueType) -> Result<()> {
        let always_block = self
            .always_stack
            .pop()
            .ok_or(CompileError::EmptyStack(String::from("always_stack")))?;

        self.builder.build_conditional_branch(
            condition.try_as_bool_value()?,
            self.current_basic_block,
            always_block,
        );

        self.builder.position_at_end(always_block);
        self.current_basic_block = always_block;
        Ok(())
    }

    fn add_terminating_instruction(
        &mut self,
        block: BasicBlock<'ctx>,
        always_block: BasicBlock<'ctx>,
    ) {
        self.builder.position_at_end(block);
        self.builder.build_unconditional_branch(always_block);
    }
}
