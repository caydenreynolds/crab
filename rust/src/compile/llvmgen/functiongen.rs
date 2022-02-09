use crate::compile::llvmgen::crab_value_type::CrabValueType;
use crate::compile::{Result, CompileError};
use crate::parse::ast::{Assignment, CodeBlock, CrabType, DoWhileStmt, ElseStmt, Expression, FnCall, FnParam, Ident, IfStmt, Primitive, Statement, StructInit, WhileStmt};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue};
use inkwell::AddressSpace;
use log::trace;
use std::collections::HashMap;
use crate::compile::llvmgen::{FnManager, StructManager};

pub struct Functiongen<'a, 'ctx> {
    builder: Builder<'ctx>,
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    fns: FnManager,
    structs: StructManager,
    variables: HashMap<Ident, CrabValueType<'ctx>>,
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
                let variables = HashMap::new();
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

                // Add variables
                // Let's just immediately store them, because that's an easy way to make the types work out
                let mut n = 0;
                for arg in args {
                    s.build_create_var(&arg.name, &arg.crab_type)?;
                    s.build_set_var(&arg.name, &CrabValueType::from_basic_value_enum(fn_value.get_nth_param(n).ok_or(CompileError::Internal(format!("Failed to get function because the param count did not match the expected number of params. i = {0}, fn_name = {1}", n, name)))?, arg.crab_type.clone()))?;
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
            Statement::FN_CALL(fc) => {self.build_fn_call(&fc)?;},
            Statement::ASSIGNMENT(ass) => self.build_assignment(ass)?,
            Statement::REASSIGNMENT(reass) => self.build_reassignment(reass)?,
            Statement::RETURN(ret) => self.build_return(ret)?,
        };
        Ok(())
    }

    fn build_assignment(&mut self, ass: &Assignment) -> Result<()> {
        let bv = self.build_expression(&ass.expr)?;
        self.build_create_var(&ass.var_name, &bv.get_crab_type())?;
        self.build_set_var(&ass.var_name, &bv)?;
        Ok(())
    }

    fn build_reassignment(&mut self, ass: &Assignment) -> Result<()> {
        let bv = self.build_expression(&ass.expr)?;
        self.build_set_var(&ass.var_name, &bv)?;
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
            Expression::FN_CALL(fc) => self.build_fn_call(fc),
            Expression::STRUCT_INIT(si) => self.build_struct_init(si),
            Expression::PRIM(prim) => self.build_primitive(prim),
            Expression::VARIABLE(var) => self.build_retrieve_var(var),
        }
    }

    fn build_struct_init(&mut self, si: &StructInit) -> Result<CrabValueType<'ctx>> {
        let crab_struct = self.structs.get(&si.name)?.clone();
        let st = self.module.get_struct_type(&crab_struct.name).ok_or(CompileError::StructDoesNotExist(si.name.clone()))?;

        if crab_struct.fields.len() != si.fields.len() {
            return Err(CompileError::StructInitFieldCount(si.name.clone(), crab_struct.fields.len(), si.fields.len()));
        }

        let mut field_vals = HashMap::new();
        for field in &si.fields {
            let val = self.build_expression(&field.value)?;
            field_vals.insert(field.name.clone(), val);
        }

        let mut init_field_list = vec![];
        for field in &crab_struct.fields {
            let val = field_vals.get(&field.name).ok_or(CompileError::StructInitFieldName(si.name.clone(), field.name.clone()))?;
            init_field_list.push(val.get_as_basic_value().ok_or(CompileError::InvalidNoneOption(String::from("build_struct_init")))?);
        }

        Ok(CrabValueType::new_struct(st.const_named_struct(&init_field_list), si.name.clone()))
    }

    fn build_primitive(&mut self, prim: &Primitive) -> Result<CrabValueType<'ctx>> {
        return match prim {
            Primitive::STRING(str) => self.build_const_string(str),
            Primitive::BOOL(bl) => Ok(self.build_const_bool(*bl)),
            Primitive::UINT(uint) => Ok(self.build_const_u64(*uint)),
        }
    }

    fn build_fn_call(&mut self, call: &FnCall) -> Result<CrabValueType<'ctx>> {
        trace!("Building a call to function {}", call.name);
        let fn_header = self.fns.get(&call.name)?.clone();

        // Check to make sure we have exactly the arguments we expect
        if call.unnamed_args.len() != fn_header.unnamed_params.len() {
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
                    args.push(self.build_expression(&named_arg.expr)?.try_as_basic_metadata_value()?);
                }
            }

            if !arg_found {
                args.push(self.build_expression(&named_param.expr)?.try_as_basic_metadata_value()?);
            }
        }

        // Build the IR
        let fn_value = self.module.get_function(&fn_header.name).ok_or(CompileError::CouldNotFindFunction(call.name.clone()))?;
        let csv = self.builder.build_call(fn_value, &args, "call");

        Ok(CrabValueType::new_call_value(csv, fn_header.return_type.clone()))
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

    fn build_create_var(&mut self, name: &Ident, expr_type: &CrabType) -> Result<()> {
        trace!("Creating a new variable with name {}", name);
        let val_ptr_result = self.variables.get(name);
        let val_ptr = match val_ptr_result {
            None => match expr_type {
                CrabType::UINT => self
                    .builder
                    .build_alloca(self.context.i64_type(), name),
                CrabType::STRING => self
                    .builder
                    .build_alloca(self.context.i8_type().ptr_type(AddressSpace::Generic), name),
                CrabType::BOOL => self
                    .builder
                    .build_alloca(self.context.custom_width_int_type(1), name),
                CrabType::STRUCT(struct_name) => {
                    let st = self.module.get_struct_type(struct_name).ok_or(CompileError::StructDoesNotExist(struct_name.clone()))?;
                    self.builder.build_alloca(st, name)
                }
                _ => unimplemented!(),
            },
            Some(_) => return Err(CompileError::VarAlreadyExists(name.clone())),
        };
        self.variables
            .insert(name.clone(), CrabValueType::new_ptr(val_ptr, expr_type.clone()));
        Ok(())
    }

    fn build_set_var(&mut self, name: &Ident, value: &CrabValueType) -> Result<()> {
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

    fn build_retrieve_var(&mut self, name: &Ident) -> Result<CrabValueType<'ctx>> {
        trace!("Retreiving a variable with name {}", name);
        match self.variables.get(name) {
            Some(var_val) => match var_val.get_crab_type() {
                CrabType::UINT => Ok(CrabValueType::new_uint(
                    self.builder
                        .build_load(var_val.try_as_ptr_value()?, name)
                        .into_int_value(),
                )),
                CrabType::STRING => Ok(CrabValueType::new_string(
                    self.builder
                        .build_load(var_val.try_as_ptr_value()?, name)
                        .into_pointer_value(),
                )),
                CrabType::BOOL => Ok(CrabValueType::new_bool(
                    self.builder
                        .build_load(var_val.try_as_ptr_value()?, name)
                        .into_int_value(),
                )),
                _ => unimplemented!(),
            },
            None => Err(CompileError::VarDoesNotExist(name.clone())),
        }
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

        self.builder
            .position_at_end(then_block);
        self.current_basic_block = then_block;

        Ok(())
    }

    fn begin_if_else(&mut self) -> Result<()> {
        // Only build a branch to end instruction if this block does not return
        if !self.codeblock_returns {
            let always_block = self.always_stack.pop().ok_or(CompileError::EmptyStack(String::from("Always stack")))?;
            self.add_terminating_instruction(self.current_basic_block, always_block);
            self.always_stack.push(always_block);
        }

        let else_block = self
            .else_stack
            .pop()
            .ok_or(CompileError::EmptyStack(String::from("else_stack")))?;
        self.builder
            .position_at_end(else_block);
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
        let then_block = self
            .context
            .append_basic_block(self.fn_value, "while");
        let always_block = self
            .context
            .append_basic_block(self.fn_value, "always");
        self.always_stack.push(always_block);
        self.builder.build_conditional_branch(condition.try_as_bool_value()?, then_block, always_block);
        self.builder.position_at_end(then_block);
        self.current_basic_block = then_block;
        Ok(())
    }

    fn begin_do_while(&mut self) -> Result<()> {
        let then_block = self
            .context
            .append_basic_block(self.fn_value, "do_while");
        let always_block = self
            .context
            .append_basic_block(self.fn_value, "always");
        self.always_stack.push(always_block);
        self.builder.build_unconditional_branch(then_block);
        self.builder.position_at_end(then_block);
        self.current_basic_block = then_block;
        Ok(())
    }

    /// Used to terminate a while or do while block
    fn end_while(&mut self, condition: &CrabValueType) -> Result<()> {
        let always_block = self.always_stack.pop().ok_or(CompileError::EmptyStack(String::from("always_stack")))?;

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
