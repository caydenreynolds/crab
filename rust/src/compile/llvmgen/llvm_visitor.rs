use crate::compile::except::{CompileError, Result};
use crate::compile::llvmgen::crab_value_type::CrabValueType;
use crate::compile::llvmgen::{Codegen, Functiongen};
use crate::compile::AstVisitor;
use crate::parse::{Assignment, AstNode, CodeBlock, CrabAst, CrabType, FnCall, Func, FuncSignature, Ident, IfStmt, Primitive, Statement, TypedIdent, TypedIdentList, WhileStmt};
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::support::LLVMString;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct LlvmVisitor<'ctx> {
    codegen: Codegen<'ctx>,
    funcgen: Option<Functiongen<'ctx>>,
    prev_basic_value: Option<CrabValueType<'ctx>>,
    return_type: Option<CrabType>,
    functions: HashMap<Ident, FuncSignature>,
    block_has_return: bool,
}

impl<'ctx> LlvmVisitor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            codegen: Codegen::new(context),
            funcgen: None,
            prev_basic_value: None,
            return_type: None,
            functions: HashMap::new(),
            block_has_return: false,
        }
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.codegen.print_to_file(path)
    }

    fn validate_return_type(&self) -> Result<()> {
        if let Some(bv) = &self.prev_basic_value {
            if let Some(rt) = &self.return_type {
                let ct = bv.get_crab_type();
                if ct == *rt {
                    Ok(())
                } else {
                    Err(CompileError::InvalidReturn(*rt, ct))
                }
            } else {
                Err(CompileError::InvalidNoneOption(String::from(
                    "validate_return_type",
                )))
            }
        } else {
            Err(CompileError::InvalidNoneOption(String::from(
                "validate_return_type",
            )))
        }
    }

    /// Get a mutable reference to self's funcgen
    /// If this function fails, it returns an error with a message defined by or
    fn get_fg(&mut self, or: &str) -> Result<&mut Functiongen<'ctx>> {
        return self
            .funcgen
            .as_mut()
            .ok_or(CompileError::InvalidNoneOption(String::from(or)))
        ;
    }

    fn get_pbv(&mut self, or: &str) -> Result<&mut CrabValueType<'ctx>> {
        return self
            .prev_basic_value
            .as_mut()
            .ok_or(CompileError::InvalidNoneOption(String::from(or)))
        ;
    }

    /*
     *******************************************************************************
     *                                                                             *
     *                                BUILTINS                                     *
     *                                                                             *
     *******************************************************************************
     */
    fn add_builtin_fns(&mut self) -> Result<()> {
        //TODO: If a use writes a function with the same name as an internal builtin (e.g. prinf), it overwrites the real printf and causes llc to barf
        //TODO: Writing a function named __printf__ also causes an overwrite (I think) and this would probably be ok, except we need to make sure it's a local overwrite, not a global one. Namespacing should fix this issue.
        self.add_printf()
    }

    /// define and add the printf function to the module
    fn add_printf(&mut self) -> Result<()> {
        self.codegen.add_function(
            "printf",
            CrabType::FLOAT,
            &[TypedIdent {
                name: String::from("str"),
                crab_type: CrabType::STRING,
            }],
            true,
            Some(Linkage::External),
        )?;
        let signature = FuncSignature {
            name: Ident::from("printf"),
            return_type: CrabType::FLOAT,
            args: Some(TypedIdentList {
                typed_idents: vec![TypedIdent {
                    name: Ident::from("str"),
                    crab_type: CrabType::STRING,
                }],
            }),
        };
        self.functions.insert(Ident::from("__printf__"), signature);
        Ok(())
    }
}

/*
 *******************************************************************************
 *                                                                             *
 *                              COMPILE AST                                    *
 *                                                                             *
 *******************************************************************************
*/

impl<'ctx> AstVisitor for LlvmVisitor<'ctx> {
    fn pre_visit(&mut self, node: &dyn AstNode) -> Result<()> {
        node.pre_visit(self)?;
        Ok(())
    }

    fn visit(&mut self, node: &dyn AstNode) -> Result<()> {
        node.visit(self)?;
        node.post_visit(self)?;
        Ok(())
    }

    fn visit_CrabAst(&mut self, node: &CrabAst) -> Result<()> {
        self.add_builtin_fns()?;
        for func in &node.functions {
            self.pre_visit(func)?;
        }
        for func in &node.functions {
            self.visit(func)?;
        }
        Ok(())
    }

    fn pre_visit_Func(&mut self, node: &Func) -> Result<()> {
        self.pre_visit(&node.signature)?;
        Ok(())
    }

    fn visit_Func(&mut self, node: &Func) -> Result<()> {
        self.visit(&node.signature)?;
        self.visit(&node.body)?;
        if !self.block_has_return {
            if node.signature.return_type == CrabType::VOID {
                self.get_fg("visit_Func")?.build_return(&CrabValueType::new_void());
            } else {
                return Err(CompileError::NoReturn(node.signature.name.clone()));
            }
        }
        Ok(())
    }

    fn post_visit_Func(&mut self, _node: &Func) -> Result<()> {
        self.funcgen = None;
        self.return_type = None;
        self.block_has_return = false;
        Ok(())
    }

    fn pre_visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
        self.codegen.add_function(
            node.name.as_str(),
            node.return_type,
            node.get_args(),
            false,
            None,
        )?;
        self.functions.insert(node.name.clone(), node.clone());

        Ok(())
    }

    fn visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
        self.funcgen = Some(
            self.codegen
                .get_function(node.name.as_str(), node.get_args())?,
        );
        self.return_type = Some(node.return_type);
        Ok(())
    }

    fn visit_CodeBlock(&mut self, node: &CodeBlock) -> Result<()> {
        for stmt in &node.statements {
            self.visit(stmt)?;
            if self.block_has_return {
                return self.get_fg("visit_CodeBlock")?.build_unreachable();
            }
        }
        Ok(())
    }

    fn visit_Statement(&mut self, node: &Statement) -> Result<()> {
        if let Some(expr) = &node.expression {
            self.visit(expr)?;
        }
        self.visit(&node.statement_type)
    }

    fn post_visit_Statement(&mut self, _node: &Statement) -> Result<()> {
        self.prev_basic_value = None;
        Ok(())
    }

    fn visit_StatementType_RETURN(&mut self, _node: &bool) -> Result<()> {
        match self.prev_basic_value {
            Some(_) => {
                let pbv = self.get_pbv("visit_StatementType_RETURN")?.clone();
                self.validate_return_type()?;
                self.get_fg("visit_StatementType_RETURN")?.build_return(&pbv)
            }
            None => self.get_fg("visit_StatementType_RETURN")?.build_return(&CrabValueType::new_void()),
        }
        self.prev_basic_value = None;
        self.block_has_return = true;
        Ok(())
    }

    fn visit_StatementType_ASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
        let assignment_type = self.get_pbv("visit_StatementType_ASSIGNMENT")?.get_crab_type();
        self.get_fg("visit_StatementType_ASSIGNMENT")?.build_create_var(&node.var_name, assignment_type)?;
        self.visit(node)
    }

    fn visit_StatementType_REASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
        self.visit(node)
    }

    fn visit_StatementType_FN_CALL(&mut self, node: &FnCall) -> Result<()> {
        self.visit(node)
    }
    fn visit_StatementType_IF_STATEMENT(&mut self, node: &IfStmt) -> Result<()> {
        self.visit(node)
    }
    fn visit_StatementType_WHILE_STATEMENT(&mut self, node: &WhileStmt) -> Result<()> {
        self.visit(node)
    }

    fn post_visit_Assignment(&mut self, node: &Assignment) -> Result<()> {
        let pbv = self.get_pbv("post_visit_Assignment")?.clone();
        self.get_fg("post_visit_Assignment")?
            .build_set_var(
                &node.var_name.clone(),
                &pbv,
            )?;
        self.prev_basic_value = None;
        Ok(())
    }

    fn visit_Expression_FN_CALL(&mut self, node: &FnCall) -> Result<()> {
        self.visit(node)
    }

    fn visit_Expression_PRIM(&mut self, node: &Primitive) -> Result<()> {
        self.visit(node)?;
        Ok(())
    }

    fn visit_Expression_VARIABLE(&mut self, node: &Ident) -> Result<()> {
        self.prev_basic_value = Some(
            self.get_fg("visit_Expression_VARIABLE")?.build_retrieve_var(node)?,
        );
        Ok(())
    }

    fn visit_Primitive_UINT64(&mut self, node: &u64) -> Result<()> {
        self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_u64(*node));
        Ok(())
    }

    fn visit_Primitive_STRING(&mut self, node: &String) -> Result<()> {
        self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_string(node)?);
        Ok(())
    }

    fn visit_Primitive_BOOL(&mut self, node: &bool) -> Result<()> {
        self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_bool(*node));
        Ok(())
    }

    fn visit_FnCall(&mut self, node: &FnCall) -> Result<()> {
        let mut args = vec![];
        for arg in &node.args.expressions {
            self.visit(arg)?;
            args.push(self.get_pbv("visit_FnCall")?.clone());
        }

        let fn_header_opt = self.functions.get(&node.name);
        if let Some(fn_header) = fn_header_opt {
            // We cam't use our fancy get_fg() fn here, because reasons
            let call_value = self.funcgen.as_mut().ok_or(CompileError::InvalidNoneOption(String::from("visit_FnCall")))?.build_fn_call(
                &fn_header.name,
                args.as_slice(),
                self.codegen.get_module(),
            )?;
            self.prev_basic_value = Some(CrabValueType::new_call_value(
                call_value,
                fn_header.return_type,
            ));
        } else {
            return Err(CompileError::CouldNotFindFunction(String::from(&node.name)));
        }
        Ok(())
    }

    // NOTE: If statements do not get their own variable space. If statements behave more like python, where a variable can be declared inside the if and then used outside of it
    fn visit_IfStmt(&mut self, node: &IfStmt) -> Result<()> {
        self.visit(&node.expr)?;
        let pbv = self.get_pbv("visit_IfStmt")?.clone();
        self.get_fg("visit_IfStmt")?.begin_if_then(&pbv)?;
        self.visit(&node.then)?;
        let then_returns = self.block_has_return;
        self.block_has_return = false;

        match &node.else_stmt {
            Some(else_stmt) => {
                self.visit(else_stmt.as_ref())?;
                self.block_has_return = then_returns && self.block_has_return;
            }
            None => self.block_has_return = false,
        }

        Ok(())
    }

    fn post_visit_IfStmt(&mut self, _node: &IfStmt) -> Result<()> {
        self.get_fg("post_visit_IfStmt")?.end_if()
    }

    fn visit_ElseStmt_ELSE(&mut self, node: &CodeBlock) -> Result<()> {
        self.get_fg("visit_ElseStmt_ELSE")?.begin_if_else()?;
        self.visit(node)
    }

    fn visit_ElseStmt_ELIF(&mut self, node: &IfStmt) -> Result<()> {
        self.visit(node)
    }

    fn visit_WhileStmt(&mut self, node: &WhileStmt) -> Result<()> {
        self.get_fg("visit_WhileStmt")?.begin_while_expr()?;
        self.visit(&node.expr)?;
        let pbv = self.get_pbv("visit_WhileStmt")?.clone();
        self.get_fg("visit_WhileStmt")?.end_while_expr(&pbv)?;
        self.visit(&node.then)?;
        self.get_fg("visit_WhileStmt")?.end_while()
    }
}
