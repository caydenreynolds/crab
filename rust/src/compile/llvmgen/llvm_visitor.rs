use crate::compile::llvmgen::{Codegen, Functiongen};
use crate::compile::{AstVisitor, BasicValueType};
use crate::parse::{AstNode, CodeBlock, CrabAst, Expression, Func, FuncSignature, Primitive};
use inkwell::context::Context;
use inkwell::support::LLVMString;
use std::path::PathBuf;

pub struct LlvmVisitor<'ctx> {
    codegen: Codegen<'ctx>,
    funcgen: Option<Functiongen<'ctx>>,
    prev_basic_value: Option<BasicValueType<'ctx>>,
}

impl<'ctx> LlvmVisitor<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            codegen: Codegen::new(context),
            funcgen: None,
            prev_basic_value: None,
        }
    }

    pub fn print_to_file(&self, path: PathBuf) -> Result<(), LLVMString> {
        self.codegen.print_to_file(path)
    }
}

impl<'ctx> AstVisitor for LlvmVisitor<'ctx> {
    fn visit(&mut self, node: &dyn AstNode) {
        node.visit(self);
        node.post_visit(self);
    }

    fn visit_CrabAst(&mut self, node: &CrabAst) {
        for func in &node.functions {
            self.visit(func);
        }
    }

    fn visit_Func(&mut self, node: &Func) {
        self.visit(&node.signature);
        self.visit(&node.body);
    }

    fn post_visit_Func(&mut self, _node: &Func) {
        self.funcgen = None;
    }

    fn visit_FuncSignature(&mut self, node: &FuncSignature) {
        self.funcgen = Some(self.codegen.add_function(node.name.as_str()))
    }

    fn visit_CodeBlock(&mut self, node: &CodeBlock) {
        for stmt in &node.statements {
            self.visit(stmt);
        }
    }

    fn visit_Statement_RETURN(&mut self, node: &Option<Expression>) {
        match node {
            Some(expr) => self.visit(expr),
            None => {} //Do nothing
        }
    }

    fn post_visit_Statement_RETURN(&mut self, node: &Option<Expression>) {
        match node {
            Some(_) => self
                .funcgen
                .as_mut()
                .unwrap()
                .build_return(&self.prev_basic_value.as_ref().unwrap()),
            None => self
                .funcgen
                .as_mut()
                .unwrap()
                .build_return(&BasicValueType::None),
        }
        self.prev_basic_value = None;
    }

    fn visit_Expression_PRIM(&mut self, node: &Primitive) {
        self.visit(node);
    }

    fn visit_Primitive_UINT64(&mut self, node: &u64) {
        self.prev_basic_value = Some(BasicValueType::IntType(
            self.funcgen.as_ref().unwrap().build_const_u64(*node),
        ))
    }
}
