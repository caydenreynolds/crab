// use crate::compile::except::{CompileError, Result};
// use crate::compile::llvmgen::crab_value_type::CrabValueType;
// use crate::compile::llvmgen::{Codegen, Functiongen};
// use crate::compile::AstVisitor;
// use crate::parse::{Assignment, AstNode, CodeBlock, CrabAst, CrabType, DoWhileStmt, FnCall, FnParam, Func, FuncSignature, Ident, IfStmt, Primitive, Statement, Struct, StructFieldInit, StructInit, WhileStmt};
// use inkwell::context::Context;
// use inkwell::module::Linkage;
// use inkwell::support::LLVMString;
// use std::collections::HashMap;
// use std::path::PathBuf;
// use inkwell::types::StructType;
// use log::trace;
//
// pub struct LlvmVisitor<'ctx> {
//     codegen: Codegen<'ctx>,
//     funcgen: Option<Functiongen<'ctx>>,
//     prev_basic_value: Option<CrabValueType<'ctx>>,
//     return_type: Option<CrabType>,
//     functions: HashMap<Ident, FuncSignature>,
//     structs: HashMap<Ident, (Struct, StructType<'ctx>)>,
//     block_has_return: bool,
// }
//
// impl<'ctx> LlvmVisitor<'ctx> {
//     pub fn new(context: &'ctx Context) -> Self {
//         Self {
//             codegen: Codegen::new(context),
//             funcgen: None,
//             prev_basic_value: None,
//             return_type: None,
//             functions: HashMap::new(),
//             structs: HashMap::new(),
//             block_has_return: false,
//         }
//     }
//
//     pub fn print_to_file(&self, path: PathBuf) -> std::result::Result<(), LLVMString> {
//         self.codegen.print_to_file(path)
//     }
//
//     fn validate_return_type(&self) -> Result<()> {
//         if let Some(bv) = &self.prev_basic_value {
//             if let Some(rt) = &self.return_type {
//                 let ct = bv.get_crab_type();
//                 if ct == *rt {
//                     Ok(())
//                 } else {
//                     Err(CompileError::InvalidReturn(*rt, ct))
//                 }
//             } else {
//                 Err(CompileError::InvalidNoneOption(String::from(
//                     "validate_return_type",
//                 )))
//             }
//         } else {
//             Err(CompileError::InvalidNoneOption(String::from(
//                 "validate_return_type",
//             )))
//         }
//     }
//
//     /// Get a mutable reference to self's funcgen
//     /// If this function fails, it returns an error with a message defined by or
//     fn get_fg(&mut self, or: &str) -> Result<&mut Functiongen<'ctx>> {
//         return self
//             .funcgen
//             .as_mut()
//             .ok_or(CompileError::InvalidNoneOption(String::from(or)));
//     }
//
//     fn get_pbv(&mut self, or: &str) -> Result<&mut CrabValueType<'ctx>> {
//         return self
//             .prev_basic_value
//             .as_mut()
//             .ok_or(CompileError::InvalidNoneOption(String::from(or)));
//     }
//
//     /*
//      *******************************************************************************
//      *                                                                             *
//      *                                BUILTINS                                     *
//      *                                                                             *
//      *******************************************************************************
//      */
//     fn add_builtin_fns(&mut self) -> Result<()> {
//         //TODO: If a use writes a function with the same name as an internal builtin (e.g. prinf), it overwrites the real printf and causes llc to barf
//         //TODO: Writing a function named __printf__ also causes an overwrite (I think) and this would probably be ok, except we need to make sure it's a local overwrite, not a global one. Namespacing should fix this issue.
//         self.add_printf()
//     }
//
//     /// define and add the printf function to the module
//     fn add_printf(&mut self) -> Result<()> {
//         self.codegen.add_function(
//             "printf",
//             CrabType::FLOAT,
//             &[FnParam {
//                 name: String::from("str"),
//                 crab_type: CrabType::STRING,
//             }],
//             true,
//             Some(Linkage::External),
//         )?;
//         let signature = FuncSignature {
//             name: Ident::from("printf"),
//             return_type: CrabType::FLOAT,
//             unnamed_params: vec![FnParam {
//                 name: Ident::from("str"),
//                 crab_type: CrabType::STRING,
//             }],
//             named_params: vec![],
//         };
//         self.functions.insert(Ident::from("__printf__"), signature);
//         Ok(())
//     }
// }
//
// /*
//  *******************************************************************************
//  *                                                                             *
//  *                              COMPILE AST                                    *
//  *                                                                             *
//  *******************************************************************************
// */
//
// impl<'ctx> AstVisitor for LlvmVisitor<'ctx> {
//     fn pre_visit(&mut self, node: &dyn AstNode) -> Result<()> {
//         node.pre_visit(self)?;
//         Ok(())
//     }
//
//     fn visit(&mut self, node: &dyn AstNode) -> Result<()> {
//         node.visit(self)?;
//         node.post_visit(self)?;
//         Ok(())
//     }
//
//     fn visit_CrabAst(&mut self, node: &CrabAst) -> Result<()> {
//         self.add_builtin_fns()?;
//         for crab_struct in &node.structs {
//             self.visit(crab_struct)?;
//         }
//         for func in &node.functions {
//             self.pre_visit(func)?;
//         }
//         for func in &node.functions {
//             self.visit(func)?;
//         }
//         Ok(())
//     }
//
//     fn visit_Struct(&mut self, node: &Struct) -> Result<()> {
//         trace!("Building a struct with name {:#?}", node.name);
//         let mut fields = vec![];
//         for field in &node.fields {
//             fields.push(field.crab_type.try_as_basic_type(self.codegen.get_context())?.clone());
//         }
//
//         let st = self.codegen.build_struct_definition(&fields);
//         self.structs.insert(node.name.clone(), (node.clone(), st));
//
//         Ok(())
//     }
//
//     fn pre_visit_Func(&mut self, node: &Func) -> Result<()> {
//         self.pre_visit(&node.signature)?;
//         Ok(())
//     }
//
//     fn visit_Func(&mut self, node: &Func) -> Result<()> {
//         self.visit(&node.signature)?;
//         self.visit(&node.body)?;
//         if !self.block_has_return {
//             if node.signature.return_type == CrabType::VOID {
//                 self.get_fg("visit_Func")?
//                     .build_return(&CrabValueType::new_void());
//             } else {
//                 return Err(CompileError::NoReturn(node.signature.name.clone()));
//             }
//         }
//         Ok(())
//     }
//
//     fn post_visit_Func(&mut self, _node: &Func) -> Result<()> {
//         self.funcgen = None;
//         self.return_type = None;
//         self.block_has_return = false;
//         Ok(())
//     }
//
//     fn pre_visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
//         self.codegen.add_function(
//             node.name.as_str(),
//             node.return_type,
//             &node.get_params(),
//             false,
//             None,
//         )?;
//         self.functions.insert(node.name.clone(), node.clone());
//
//         Ok(())
//     }
//
//     fn visit_FuncSignature(&mut self, node: &FuncSignature) -> Result<()> {
//         self.funcgen = Some(
//             self.codegen
//                 .get_function(node.name.as_str(), &node.get_params())?,
//         );
//         self.return_type = Some(node.return_type);
//         Ok(())
//     }
//
//     fn visit_CodeBlock(&mut self, node: &CodeBlock) -> Result<()> {
//         for stmt in &node.statements {
//             self.visit(stmt)?;
//             if self.block_has_return {
//                 return self.get_fg("visit_CodeBlock")?.build_unreachable();
//             }
//         }
//         Ok(())
//     }
//
//     fn visit_Statement(&mut self, node: &Statement) -> Result<()> {
//         if let Some(expr) = &node.expression {
//             self.visit(expr)?;
//         }
//         self.visit(&node.statement_type)
//     }
//
//     fn post_visit_Statement(&mut self, _node: &Statement) -> Result<()> {
//         self.prev_basic_value = None;
//         Ok(())
//     }
//
//     fn visit_StatementType_RETURN(&mut self, _node: &bool) -> Result<()> {
//         match self.prev_basic_value {
//             Some(_) => {
//                 let pbv = self.get_pbv("visit_StatementType_RETURN")?.clone();
//                 self.validate_return_type()?;
//                 self.get_fg("visit_StatementType_RETURN")?
//                     .build_return(&pbv)
//             }
//             None => self
//                 .get_fg("visit_StatementType_RETURN")?
//                 .build_return(&CrabValueType::new_void()),
//         }
//         self.prev_basic_value = None;
//         self.block_has_return = true;
//         Ok(())
//     }
//
//     fn visit_StatementType_ASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
//         let assignment_type = self
//             .get_pbv("visit_StatementType_ASSIGNMENT")?
//             .get_crab_type();
//         self.get_fg("visit_StatementType_ASSIGNMENT")?
//             .build_create_var(&node.var_name, assignment_type)?;
//         self.visit(node)
//     }
//
//     fn visit_StatementType_REASSIGNMENT(&mut self, node: &Assignment) -> Result<()> {
//         self.visit(node)
//     }
//
//     fn visit_StatementType_FN_CALL(&mut self, node: &FnCall) -> Result<()> {
//         self.visit(node)
//     }
//     fn visit_StatementType_IF_STATEMENT(&mut self, node: &IfStmt) -> Result<()> {
//         self.visit(node)
//     }
//     fn visit_StatementType_WHILE_STATEMENT(&mut self, node: &WhileStmt) -> Result<()> {
//         self.visit(node)
//     }
//     fn visit_StatementType_DO_WHILE_STATEMENT(&mut self, node: &DoWhileStmt) -> Result<()> {
//         self.visit(node)
//     }
//
//     fn post_visit_Assignment(&mut self, node: &Assignment) -> Result<()> {
//         let pbv = self.get_pbv("post_visit_Assignment")?.clone();
//         self.get_fg("post_visit_Assignment")?
//             .build_set_var(&node.var_name.clone(), &pbv)?;
//         self.prev_basic_value = None;
//         Ok(())
//     }
//
//     fn visit_Expression_FN_CALL(&mut self, node: &FnCall) -> Result<()> {
//         self.visit(node)
//     }
//
//     fn visit_Expression_PRIM(&mut self, node: &Primitive) -> Result<()> {
//         self.visit(node)?;
//         Ok(())
//     }
//
//     fn visit_Expression_VARIABLE(&mut self, node: &Ident) -> Result<()> {
//         self.prev_basic_value = Some(
//             self.get_fg("visit_Expression_VARIABLE")?
//                 .build_retrieve_var(node)?,
//         );
//         Ok(())
//     }
//
//     fn visit_Expression_STRUCT_INIT(&mut self, node: &StructInit) -> Result<()> {
//         let (ct, struct_type) = self.structs.get(&node.name).ok_or(CompileError::NoType(node.name.clone()))?.clone();
//
//         if node.fields.len() != ct.fields.len() {
//             return Err(CompileError::StructInitFieldCount(node.name.clone(), ct.fields.len(), node.fields.len()));
//         }
//
//         let mut field_vals = HashMap::new();
//         for field in node.fields {
//             self.visit(&field);
//             field_vals.insert(field.name.clone(), self.get_pbv("visit_Expression_STRUCT_INIT")?);
//         }
//
//         let mut init_field_list = vec![];
//         for field in ct.fields {
//             let val = field_vals.get(&field.name).ok_or(CompileError::StructInitFieldName(ct.name.clone(), field.name.clone()))?;
//             init_field_list.push(val.get_as_basic_value().ok_or(CompileError::InvalidNoneOption(String::from("visit_Expression_STRUCT_INIT")))?);
//         }
//
//         self.prev_basic_value = Some(
//             self.get_fg("visit_Expression_STRUCT_INIT")?
//                 .build_struct_init(&struct_type, &init_field_list, node.name.clone())
//         );
//         Ok(())
//     }
//
//     fn visit_StructFieldInit(&mut self, node: &StructFieldInit) -> Result<()> {
//         self.visit(&node.value)
//     }
//
//     fn visit_Primitive_UINT64(&mut self, node: &u64) -> Result<()> {
//         self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_u64(*node));
//         Ok(())
//     }
//
//     fn visit_Primitive_STRING(&mut self, node: &String) -> Result<()> {
//         self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_string(node)?);
//         Ok(())
//     }
//
//     fn visit_Primitive_BOOL(&mut self, node: &bool) -> Result<()> {
//         self.prev_basic_value = Some(self.funcgen.as_ref().unwrap().build_const_bool(*node));
//         Ok(())
//     }
//
//     fn visit_FnCall(&mut self, node: &FnCall) -> Result<()> {
//         let fn_header = self
//             .functions
//             .get(&node.name)
//             .ok_or(CompileError::CouldNotFindFunction(String::from(&node.name)))?
//             .clone();
//
//         // Check to make sure we have exactly the arguments we expect
//         if node.unnamed_args.len() != fn_header.unnamed_params.len() {
//             return Err(CompileError::PositionalArgumentCount(
//                 fn_header.name.clone(),
//                 fn_header.unnamed_params.len(),
//                 node.unnamed_args.len(),
//             ));
//         }
//         for named_expr in &node.named_args {
//             if !fn_header
//                 .named_params
//                 .iter()
//                 .any(|param| param.name == named_expr.name)
//             {
//                 return Err(CompileError::InvalidNamedArgument(
//                     fn_header.name.clone(),
//                     named_expr.name.clone(),
//                 ));
//             }
//         }
//
//         let mut args = vec![];
//
//         // Handle all of the positional arguments
//         for arg in &node.unnamed_args {
//             self.visit(arg)?;
//             args.push(self.get_pbv("visit_FnCall")?.clone());
//         }
//
//         // Handle all of the optional arguments
//         for named_param in fn_header.named_params {
//             let mut arg_found = false;
//             for named_arg in &node.named_args {
//                 if named_param.name == named_arg.name {
//                     arg_found = true;
//                     self.visit(&named_arg.expr)?;
//                     args.push(self.get_pbv("visit_FnCall")?.clone());
//                 }
//             }
//
//             if !arg_found {
//                 self.visit(&named_param.expr)?;
//                 args.push(self.get_pbv("visit_FnCall")?.clone());
//             }
//         }
//
//         // We can't use our fancy get_fg() fn here, because reasons
//         let call_value = self
//             .funcgen
//             .as_mut()
//             .ok_or(CompileError::InvalidNoneOption(String::from(
//                 "visit_FnCall",
//             )))?
//             .build_fn_call(
//                 &fn_header.name.clone(),
//                 args.as_slice(),
//                 self.codegen.get_module(),
//             )?;
//         self.prev_basic_value = Some(CrabValueType::new_call_value(
//             call_value,
//             fn_header.return_type,
//         ));
//         Ok(())
//     }
//
//     // NOTE: If statements do not get their own variable space. If statements behave more like python, where a variable can be declared inside the if and then used outside of it
//     fn visit_IfStmt(&mut self, node: &IfStmt) -> Result<()> {
//         self.visit(&node.expr)?;
//         let pbv = self.get_pbv("visit_IfStmt")?.clone();
//         self.get_fg("visit_IfStmt")?.begin_if_then(&pbv)?;
//         self.visit(&node.then)?;
//         let then_returns = self.block_has_return;
//         self.block_has_return = false;
//
//         match &node.else_stmt {
//             Some(else_stmt) => {
//                 self.visit(else_stmt.as_ref())?;
//                 self.block_has_return = then_returns && self.block_has_return;
//             }
//             None => self.block_has_return = false,
//         }
//
//         Ok(())
//     }
//
//     fn post_visit_IfStmt(&mut self, _node: &IfStmt) -> Result<()> {
//         self.get_fg("post_visit_IfStmt")?.end_if()
//     }
//
//     fn visit_ElseStmt_ELSE(&mut self, node: &CodeBlock) -> Result<()> {
//         self.get_fg("visit_ElseStmt_ELSE")?.begin_if_else()?;
//         self.visit(node)
//     }
//
//     fn visit_ElseStmt_ELIF(&mut self, node: &IfStmt) -> Result<()> {
//         self.visit(node)
//     }
//
//     fn visit_WhileStmt(&mut self, node: &WhileStmt) -> Result<()> {
//         self.visit(&node.expr)?;
//         let pbv = self.get_pbv("visit_WhileStmt")?.clone();
//         self.get_fg("visit_WhileStmt")?.begin_while(&pbv)?;
//
//         self.visit(&node.then)?;
//         self.visit(&node.expr)?;
//         let pbv = self.get_pbv("visit_WhileStmt")?.clone();
//         self.get_fg("visit_WhileStmt")?.end_while(&pbv)
//     }
//
//     fn visit_DoWhileStmt(&mut self, node: &DoWhileStmt) -> Result<()> {
//         self.get_fg("visit_DoWhileStmt")?.begin_do_while()?;
//         self.visit(&node.then)?;
//         self.visit(&node.expr)?;
//         let pbv = self.get_pbv("visit_WhileStmt")?.clone();
//         self.get_fg("visit_DoWhileStmt")?.end_while(&pbv)
//     }
// }
