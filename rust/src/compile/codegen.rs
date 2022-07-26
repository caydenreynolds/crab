use crate::compile::{
    add_builtin_definition, add_main_func, CompileError, FnManager, Result, TypeManager, VarManager,
};
use crate::parse::ast::{Assignment, CodeBlock, CrabAst, CrabType, DoWhileStmt, Expression, ExpressionType, FnBodyType, FnCall, Ident, IfStmt, PosParam, Primitive, Statement, StructId, StructInit, WhileStmt};
use crate::quill::{ArtifactType, ChildNib, FnNib, Nib, PolyQuillType, Quill, QuillBoolType, QuillFnType, QuillPointerType, QuillStructType, QuillType, QuillValue};
use crate::util::{
    mangle_function_name, primitive_field_name, ListFunctional, MapFunctional, SetFunctional,
};
use log::{debug, trace};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::path::Path;
use std::rc::Rc;

///
/// Compiles the given CrabAst and writes the output to out_path
///
/// Params:
/// * `ast` - The CrabAst to compile
/// * `out_path` - The path to write the output to
/// * `artifact_type` - The type of artifact to output
///
pub fn compile(
    ast: CrabAst,
    out_path: &Path,
    artifact_type: &ArtifactType,
    verify: bool,
) -> Result<()> {
    trace!("Called parse::compile");
    let mut peter: Quill = Quill::new();
    let mut type_manager = TypeManager::new();

    ast.structs
        .into_iter()
        .try_for_each(|crab_struct| type_manager.register_struct(crab_struct))?;
    ast.interfaces
        .into_iter()
        .try_for_each(|(_, crab_interface)| type_manager.register_interface(crab_interface))?;
    ast.intrs
        .into_iter()
        .try_for_each(|crab_intr| type_manager.register_intr(crab_intr))?;

    let type_manager = Rc::new(RefCell::new(type_manager));
    let fn_manager = Rc::new(RefCell::new(FnManager::new(type_manager.clone())));

    ast.functions
        .into_iter()
        .for_each(|func| fn_manager.borrow_mut().add_source(func));
    fn_manager.borrow_mut().add_main_to_queue()?;

    while !fn_manager.borrow_mut().build_queue_empty() {
        let func = fn_manager.borrow_mut().pop_build_queue().unwrap();
        let name = func.signature.name.clone();
        debug!("Building function with name {}", name);
        let fn_t = type_manager
            .borrow_mut()
            .get_quill_fn_type(func.signature.clone())?;
        let mut nib = FnNib::new(name.clone(), fn_t);
        let (nib, returns) = match func.body {
            FnBodyType::CODEBLOCK(cb) => {
                let all_params =
                    func.signature
                        .pos_params
                        .into_iter()
                        .chain(func.signature.named_params.into_iter().map(|named_param| {
                            PosParam {
                                name: named_param.name,
                                crab_type: named_param.crab_type,
                            }
                        }))
                        .collect();
                let mut codegen =
                    Codegen::new(nib, type_manager.clone(), fn_manager.clone(), all_params)?;
                let returns = codegen.build_codeblock(cb)?;
                (codegen.into_nib(), returns)
            }
            FnBodyType::COMPILER_PROVIDED => {
                add_builtin_definition(&mut peter, &mut nib)?;
                (nib, true) // Just assume it's all good for now
            }
        };

        match returns {
            true => peter.add_fn(nib),
            false => return Err(CompileError::NoReturn(name)),
        }
    }

    let mut tm = type_manager.borrow_mut();
    tm.get_included_type_names()
        .clone()
        .into_iter()
        .try_for_each(|crab_struct| {
            let name = crab_struct.id.name.clone();
            peter.register_struct_type(name.id.name.clone(), tm.get_fields(&crab_struct)?);
            Result::Ok(())
        })?;
    add_main_func(&mut peter)?;
    peter.commit(out_path, &artifact_type, verify)?;
    Ok(())
}

struct Codegen<NibType: Nib> {
    nib: NibType,
    vars: VarManager,
    types: Rc<RefCell<TypeManager>>,
    fns: Rc<RefCell<FnManager>>,
}
impl<NibType: Nib> Codegen<NibType> {
    ///
    /// Creates a new Codegen, which has its own ChildNib and inherits everything else
    ///
    fn create_child(&self) -> Codegen<ChildNib> {
        Codegen {
            nib: self.nib.create_child(),
            vars: self.vars.clone(),
            types: self.types.clone(),
            fns: self.fns.clone(),
        }
    }

    fn into_nib(self) -> NibType {
        self.nib
    }

    ///
    /// Build a Nib for a given codeblock
    ///
    /// Params:
    /// * `codeblock` - The codeblock to build
    ///
    /// Returns:
    /// This codeblock's Nib, with all of the required statements added to it
    /// True if the built codeblock will always return a value, or false otherwise
    ///
    fn build_codeblock(&mut self, codeblock: CodeBlock) -> Result<bool> {
        trace!("Codegen::build_codeblock");
        let returns = codeblock
            .statements
            .into_iter()
            .try_fold(false, |returns, stmt| {
                if returns {
                    Ok(true)
                } else {
                    self.build_statement(stmt)
                }
            })?;
        Ok(returns)
    }

    ///
    /// Adds a given statement to the Nib
    ///
    /// Params:
    /// * `stmt` - The statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_statement(&mut self, stmt: Statement) -> Result<bool> {
        trace!("Codegen::build_statement");
        match stmt {
            Statement::IF_STATEMENT(is) => self.build_if_stmt(is),
            Statement::WHILE_STATEMENT(ws) => self.build_while_statement(ws),
            Statement::DO_WHILE_STATEMENT(dws) => self.build_do_while_statement(dws),
            Statement::EXPRESSION(expr) => self.build_expression(expr, None).map(|_| false),
            Statement::ASSIGNMENT(ass) => self.build_assignment(ass),
            Statement::REASSIGNMENT(reass) => self.build_reassignment(reass),
            Statement::RETURN(ret) => self.build_return(ret),
        }
    }

    ///
    /// Adds the given return statement to the Nib
    /// Yes, this function always returns true
    ///
    /// Params:
    /// * `expr` - The optional expression to return
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_return(&mut self, ret: Option<Expression>) -> Result<bool> {
        trace!("Codegen::build_return");
        match ret {
            None => self.nib.add_return(QuillFnType::void_return_value()),
            Some(expr) => {
                let expr_res = self.build_expression(expr, None)?;
                self.nib.add_return(Some(&expr_res.quill_value));
            }
        }
        Ok(true)
    }

    ///
    /// Builds the given assignment statement
    /// Keeps a local copy of a value, by name
    /// Adds any necessary expression to the Nib
    /// This function always returns false
    ///
    /// Params:
    /// * `ass` - The assignment to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_assignment(&mut self, ass: Assignment) -> Result<bool> {
        trace!("Codegen::build_assignment");
        let value = self.build_expression(ass.expr, None)?;
        self.vars.assign(ass.var_name, value)?;
        Ok(false)
    }

    ///
    /// Builds the given reassignment statement
    /// Keeps a local copy of a value, by name
    /// Any previous value will be cleared
    /// Adds any necessary expression to the Nib
    /// This function always returns false
    ///
    /// Params:
    /// * `reass` - The assignment to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_reassignment(&mut self, reass: Assignment) -> Result<bool> {
        trace!("Codegen::build_reassignment");
        let value = self.build_expression(reass.expr, None)?;
        self.vars.reassign(reass.var_name, value)?;
        Ok(false)
    }

    ///
    /// Adds a given if statement to the Nib
    ///
    /// Params:
    /// * `is` - The if statement to build
    ///
    /// Returns:
    /// True if the statement always returns a value, or false otherwise
    ///
    fn build_if_stmt(&mut self, is: IfStmt) -> Result<bool> {
        trace!("Codegen::build_if_stmt");
        // Build all the different blocks
        let mut then_codegen = self.create_child();
        let then_returns = then_codegen.build_codeblock(is.then)?;
        let (else_codegen, else_returns) = match is.else_stmt {
            None => (None, None),
            Some(cb) => {
                let mut else_codegen = self.create_child();
                let returns = else_codegen.build_codeblock(cb)?;
                (Some(else_codegen), Some(returns))
            }
        };

        // Build the branch statement
        let value = self.build_expression(is.expr, None)?;
        let value_value = self.nib.get_value_from_struct(
            &value.quill_value.try_into()?,
            primitive_field_name(),
            QuillBoolType::new(),
        )?;
        self.nib.add_cond_branch(
            &value_value,
            then_codegen.into_nib(),
            else_codegen.map(|ec| ec.into_nib()),
        );

        // Handle different cases that may occur with return values
        let always_returns = match else_returns {
            None => then_returns,
            Some(else_returns) => then_returns && else_returns,
        };
        if always_returns {
            self.nib.build_unreachable();
        }
        Ok(always_returns)
    }

    ///
    /// Adds a given while statement to the Nib
    ///
    /// Params:
    /// * `ws` - The while statement to build
    ///
    /// Returns:
    /// True if the while statement always returns a value, or false otherwise
    ///
    fn build_while_statement(&mut self, ws: WhileStmt) -> Result<bool> {
        trace!("Codegen::build_while_statement");
        // Build the internal codeblock
        let mut while_codegen = self.create_child();
        let while_returns = while_codegen.build_codeblock(ws.then)?;
        let value = while_codegen.build_expression(ws.expr.clone(), None)?;
        let value_value = while_codegen.nib.get_value_from_struct(
            &value.quill_value.try_into()?,
            primitive_field_name(),
            QuillBoolType::new(),
        )?;
        let mut while_nib = while_codegen.into_nib();
        while_nib.add_cond_loop(&value_value);

        // Build our entrypoint into the while codeblock
        let value = self.build_expression(ws.expr, None)?;
        let value_value = self.nib.get_value_from_struct(
            &value.quill_value.try_into()?,
            primitive_field_name(),
            QuillBoolType::new(),
        )?;
        self.nib.add_cond_branch(&value_value, while_nib, None);

        if while_returns {
            self.nib.build_unreachable();
        }
        Ok(while_returns)
    }

    ///
    /// Adds a given do-while statement to the Nib
    ///
    /// Params:
    /// * `dws` - The while statement to build
    ///
    /// Returns:
    /// True if the while statement always returns a value, or false otherwise
    ///
    fn build_do_while_statement(&mut self, dws: DoWhileStmt) -> Result<bool> {
        trace!("Codegen::build_do_while_statement");
        // Build the internal codeblock
        let mut do_while_codegen = self.create_child();
        let do_while_returns = do_while_codegen.build_codeblock(dws.then)?;
        let value = do_while_codegen.build_expression(dws.expr, None)?;
        let value_value = do_while_codegen.nib.get_value_from_struct(
            &value.quill_value.try_into()?,
            primitive_field_name(),
            QuillBoolType::new(),
        )?;

        let mut do_while_nib = do_while_codegen.into_nib();
        do_while_nib.add_cond_loop(&value_value);
        // Build our entrypoint into the do-while codeblock
        self.nib.add_branch(do_while_nib);

        if do_while_returns {
            self.nib.build_unreachable();
        }
        Ok(do_while_returns)
    }

    ///
    /// Adds the given expression to the Nib
    ///
    /// Params:
    /// * `expr` - The expression to build
    /// * `prev` - The previous value in the expression chain
    ///
    /// Returns:
    /// The resultant value of the expression
    ///
    fn build_expression(
        &mut self,
        expr: Expression,
        prev: Option<CrabValue>,
    ) -> Result<CrabValue> {
        trace!("Codegen::build_expression");
        let val = match expr.this {
            ExpressionType::PRIM(prim) => Ok(self.build_primitive(prim)),
            ExpressionType::STRUCT_INIT(si) => Ok(self.build_struct_init(si)?),
            ExpressionType::FN_CALL(fc) => self.build_fn_call(fc, prev),
            ExpressionType::VARIABLE(id) => {
                match prev {
                    None => self.vars.get(&id)?.clone(),
                    Some(prev) => {
                        // Figure out what type of value we should get from the struct
                        let prev_strct = match prev.quill_value.get_type() {
                            PolyQuillType::PointerType(pst) => {
                                QuillStructType::try_from(pst.get_inner_type())?
                            }
                            t => {
                                return Err(CompileError::NotAStruct(
                                    StructId::from_name(Ident::from("Some invalid PolyQuillType")),
                                    String::from("Codegen::build_expression"),
                                ))
                            }
                        };
                        let expected_type = self
                            .types
                            .borrow_mut()
                            .get_fields(&prev.crab_type)?
                            .iter()
                            .filter(|(name, _)| **name == id)
                            .next()
                            .map(|(_, pqt)| pqt.clone())
                            .ok_or(CompileError::StructFieldName(
                                prev.crab_type.clone(),
                                prev_strct.get_name(),
                            ))?;

                        // Get that value from the struct
                        let val = self.nib.get_value_from_struct(
                            &prev.quill_value.try_into()?,
                            id,
                            expected_type.clone(),
                        )?;
                        let expected_ct = self
                            .types
                            .borrow_mut()
                            .get_field_types(&prev.crab_type)?
                            .iter()
                            .filter(|(name, _)| **name == id)
                            .next()
                            .ok_or(CompileError::StructFieldName(prev.crab_type.clone(), id.clone()))?
                            .1
                            .clone();
                        Ok(CrabValue::new(val, expected_ct))
                    }
                }
            }
        }?;

        match expr.next {
            None => Ok(val),
            Some(next) => self.build_expression(*next, Some(val)),
        }
    }

    ///
    /// Gets a quill value for the given primitive
    ///
    /// Params:
    /// * `prim` - The prim to get the quill value for
    ///
    /// Returns:
    /// The new quill value
    ///
    fn build_primitive(&mut self, prim: Primitive) -> CrabValue {
        trace!("Codegen::build_primitive");
        match prim {
            Primitive::STRING(value) => CrabValue::new(self.nib.const_string(value), CrabType::PRIM_STR),
            Primitive::BOOL(value) => CrabValue::new(self.nib.const_bool(value), CrabType::PRIM_BOOL),
            Primitive::UINT(value) => CrabValue::new(self.nib.const_int(64, value), CrabType::PRIM_INT),
        }
    }

    ///
    /// Adds a struct initialization to the Nib
    ///
    /// Params:
    /// * `si` - The struct init to add
    ///
    /// Returns:
    /// The value of the new struct
    ///
    fn build_struct_init(&mut self, si: StructInit) -> Result<CrabValue> {
        trace!("Codegen::build_struct_init");
        let struct_id = si.id;
        let struct_field_names = self
            .types
            .borrow_mut()
            .get_fields(&struct_id)?
            .iter()
            .fold(HashSet::new(), |struct_field_names, (name, _)| {
                struct_field_names.finsert(name.clone())
            });

        let fields =
            si.fields
                .into_iter()
                .try_fold(HashMap::new(), |field_vals, field| match struct_field_names
                    .get(&field.name)
                {
                    Some(_) => {
                        Ok(field_vals
                            .finsert(field.name, self.build_expression(field.value, None)?))
                    }
                    None => Err(CompileError::StructFieldName(
                        struct_id.clone(),
                        field.name,
                    )),
                })?;

        struct_field_names
            .into_iter()
            .try_for_each(|name| match fields.contains_key(&name) {
                true => Ok(()),
                false => Err(CompileError::StructInitFieldName(struct_name.clone(), name)),
            })?;

        //TODO: free
        let struct_t = self.types.borrow_mut().get_quill_struct(&struct_id)?;
        let new_struct_ptr = self.nib.add_malloc(struct_t);
        fields.into_iter().try_for_each(|(name, value)| {
            self.nib.set_value_in_struct(&new_struct_ptr, name, value.quill_value)
        })?;

        Ok(CrabValue::new(new_struct_ptr, struct_id))
    }

    fn build_fn_call(
        &mut self,
        call: FnCall,
        caller_opt: Option<CrabValue>,
    ) -> Result<CrabValue> {
        trace!("Codegen::build_fn_call");
        // Get the original function
        // TODO: Once we have namespaces and stuff, we should only be manging inside fn_manager
        let caller_ct = caller_opt.map(|caller| &caller.crab_type);
        let source_signature = self.fns.borrow_mut().get_source_signature(&call.name, caller_ct)?;

        // Handle all of the positional arguments
        let unnamed_args =
            call.pos_args
                .iter()
                .try_fold(vec![], |unnamed_args, unnamed_arg| {
                    Result::Ok(
                        unnamed_args.fpush(self.build_expression(unnamed_arg.clone(), None)?),
                    )
                })?;

        // Handle all of the optional arguments
        // First add all of the args that were supplied in the ast
        // Then, for any args that are missing from the ast, build the expressions in the source_signature to fill in the gaps
        let named_args =
            call.named_args
                .iter()
                .try_fold(BTreeMap::new(), |named_args, named_arg| {
                    Result::Ok(named_args.finsert(
                        named_arg.name.clone(),
                        self.build_expression(named_arg.expr.clone(), None)?,
                    ))
                })?;
        let named_args = source_signature.named_params.into_iter().try_fold(
            named_args,
            |named_args, (_, named_param)| match named_args.get(&named_param.name) {
                Some(_) => Result::Ok(named_args),
                None => Result::Ok(named_args.finsert(
                    named_param.name,
                    self.build_expression(named_param.expr, None)?,
                )),
            },
        )?;

        // The function we're actually calling will be different for different argument types
        // So we need to get the signature of the method we actually want to call
        let signature =
            self.fns
                .borrow_mut()
                .get_signature(&call, caller_ct, &unnamed_args, &named_args)?;

        // Listify the named params in the correct order
        let quill_fn_t = self
            .types
            .borrow_mut()
            .get_quill_fn_type(signature.clone())?;
        let args =
            quill_fn_t
                .get_params()
                .iter()
                .enumerate()
                .fold(vec![], |args, (i, (name, _))| {
                    if i < unnamed_args.len() {
                        args.fpush(unnamed_args.get(i).unwrap().clone())
                    } else {
                        args.fpush(named_args.get(name).unwrap().clone())
                    }
                });
        let qv = self.nib.add_fn_call(
            signature.name,
            args.into_iter().map(|cv| cv.quill_value.clone()).collect(),
            self.types
                .borrow_mut()
                .get_quill_type(&signature.return_type)?,
        );
        Ok(CrabValue::new(qv, signature.return_type))
    }
}

impl Codegen<FnNib> {
    ///
    /// Creates a new Codegen, with an empty set of variables
    ///
    /// Params:
    /// * `nib` - The nib to build everything into
    /// * `types` - The TypeManager to use for resolving types
    ///
    fn new(
        mut nib: FnNib,
        types: Rc<RefCell<TypeManager>>,
        fns: Rc<RefCell<FnManager>>,
        fn_params: Vec<PosParam>,
    ) -> Result<Self> {
        let mut vars = VarManager::new();
        fn_params.into_iter().try_for_each(|fn_param| {
            let val = nib.get_fn_param(
                fn_param.name.clone(),
                types.borrow_mut().get_quill_type(&fn_param.crab_type)?,
            );
            vars.assign(fn_param.name, CrabValue::new(val.into(), fn_param.crab_type)).unwrap();
            Result::Ok(())
        })?;
        Ok(Self {
            nib,
            types,
            fns,
            vars,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CrabValue {
    pub quill_value: QuillValue<PolyQuillType>,
    pub crab_type: CrabType,
}

impl CrabValue {
    pub fn new<T: QuillType>(qv: QuillValue<T>, ct: CrabType) -> Self {
        Self {
            quill_value: qv.into(),
            crab_type: ct,
        }
    }
}
