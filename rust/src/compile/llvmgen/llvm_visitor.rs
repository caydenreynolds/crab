use crate::compile::except::{CompileError, Result};
use crate::compile::llvmgen::{Codegen, Functiongen};
use crate::compile::{AstVisitor, BasicValueType};
use crate::parse::{
    Assignment, AstNode, CodeBlock, CrabAst, CrabType, FnCall, Func, FuncSignature, Ident,
    Primitive, Statement,
};
use inkwell::context::Context;
use inkwell::support::LLVMString;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct LlvmVisitor<'ctx> {
    codegen: Codegen<'ctx>,
    funcgen: Option<Functiongen<'ctx>>,
    prev_basic_value: Option<BasicValueType<'ctx>>,
    return_type: Option<CrabType>,
    functions: HashMap<Ident, FuncSignature>,
}

impl<'ctx> LlvmVisitor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            codegen: Codegen::new(context),
            funcgen: None,
            prev_basic_value: None,
            return_type: None,
            functions: HashMap::new(),
        }
    }

    pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
        self.codegen.print_to_file(path)
    }

    fn validate_return_type(&self) -> Result<()> {
        if let Some(bv) = &self.prev_basic_value {
            if let Some(rt) = &self.return_type {
                let ct = bv.to_crab_type();
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
}

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
        Ok(())
    }

    fn post_visit_Func(&mut self, _node: &Func) -> Result<()> {
        self.funcgen = None;
        self.return_type = None;
        Ok(())
    }

    fn pre_visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
        self.codegen.add_function(node.name.as_str());
        self.functions.insert(node.name.clone(), node.clone());

        Ok(())
    }

    fn visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
        self.funcgen = Some(self.codegen.get_function(node.name.as_str())?);
        self.return_type = Some(node.return_type);
        Ok(())
    }

    fn visit_CodeBlock(&mut self, node: &CodeBlock) -> Result<()> {
        for stmt in &node.statements {
            self.visit(stmt)?;
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
                self.validate_return_type()?;
                self.funcgen
                    .as_mut()
                    .unwrap()
                    .build_return(&self.prev_basic_value.as_ref().unwrap())
            }
            None => self
                .funcgen
                .as_mut()
                .unwrap()
                .build_return(&BasicValueType::None),
        }
        self.prev_basic_value = None;
        Ok(())
    }

    fn visit_StatementType_ASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
        let assignment_type = self
            .prev_basic_value
            .as_ref()
            .ok_or(CompileError::InvalidNoneOption(String::from(
                "visit_Statement_ASSIGNMENT",
            )))?
            .to_crab_type();
        self.funcgen
            .as_mut()
            .ok_or(CompileError::InvalidNoneOption(String::from(
                "visit_Statement_ASSIGNMENT",
            )))?
            .build_create_var(&node.var_name, assignment_type)?;
        self.visit(node)
    }

    fn visit_StatementType_REASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
        self.visit(node)
    }

    fn post_visit_Assignment(&mut self, node: &Assignment) -> Result<()> {
        self.funcgen
            .as_mut()
            .ok_or(CompileError::InvalidNoneOption(String::from(
                "post_visit_Assignment",
            )))?
            .build_set_var(
                &node.var_name.clone(),
                self.prev_basic_value
                    .as_ref()
                    .ok_or(CompileError::InvalidNoneOption(String::from(
                        "post_visit_Assignment",
                    )))?,
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
            self.funcgen
                .as_mut()
                .ok_or(CompileError::InvalidNoneOption(String::from(
                    "visit_Expression_VARIABLE",
                )))?
                .build_retrieve_var(node)?,
        );
        Ok(())
    }

    fn visit_Primitive_UINT64(&mut self, node: &u64) -> Result<()> {
        self.prev_basic_value = Some(BasicValueType::IntValue(
            self.funcgen.as_ref().unwrap().build_const_u64(*node),
            CrabType::UINT,
        ));
        Ok(())
    }

    fn visit_Primitive_STRING(&mut self, node: &String) -> Result<()> {
        self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_string(node));
        Ok(())
    }

    fn visit_FnCall(&mut self, node: &FnCall) -> Result<()> {
        let call_value = self
            .funcgen
            .as_mut()
            .unwrap()
            .build_fn_call(&node.name, self.codegen.get_module())?;
        let fn_header_opt = self.functions.get(&node.name);
        if let Some(fn_header) = fn_header_opt {
            self.prev_basic_value =
                Some(BasicValueType::CallValue(call_value, fn_header.return_type));
        } else {
            return Err(CompileError::CouldNotFindFunction(String::from(&node.name)));
        }
        Ok(())
    }
}
