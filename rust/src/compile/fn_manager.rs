use crate::compile::{CompileError, CrabValueType, ManagedType, Result, TypeManager};
use crate::parse::ast::{CodeBlock, CrabType, FnCall, FnParam, Func, FuncSignature, Ident};
use crate::util::{add_param_mangles, main_func_name, mangle_function_name};
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use log::trace;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct FnManager<'a, 'ctx> {
    /// All fns that have been defined in the source code
    fn_sources: HashMap<Ident, Func>,

    /// All fns that have been registered to be built
    registered_fns: HashSet<Ident>,

    /// All registered fns that have not been built yet
    fn_build_queue: Vec<Func>,

    /// All of the registered types. Required for resolving params
    types: TypeManager,

    context: &'ctx Context,
    module: &'a Module<'ctx>,
}

impl<'a, 'ctx> FnManager<'a, 'ctx> {
    pub fn new(
        types: TypeManager,
        context: &'ctx Context,
        module: &'a Module<'ctx>,
    ) -> FnManager<'a, 'ctx> {
        Self {
            fn_sources: Default::default(),
            registered_fns: Default::default(),
            fn_build_queue: vec![],
            types,
            context,
            module,
            // ..Self::default()
        }
    }

    ///
    /// Add a Func directly from the CrabAst
    /// A function that has not been added as source cannot be registered or built
    /// It is recommend to call add_source on every Func in the AST
    ///
    /// Params:
    /// * `source` - The Func to add to this FnManager's sources
    ///
    pub fn add_source(&mut self, source: Func) {
        self.fn_sources
            .insert(source.signature.name.clone(), source);
    }

    ///
    /// Adds the main function to the function build queue
    ///
    /// Returns:
    /// An error if there is no main function source
    ///
    pub fn add_main_to_queue(&mut self) -> Result<()> {
        let main = self
            .fn_sources
            .get(&mangle_function_name(&main_func_name(), None))
            .ok_or(CompileError::NoMain)?
            .clone();
        self.fn_build_queue.push(main.clone());
        self.register_function(main.signature, false, None)?;
        Ok(())
    }

    ///
    /// Removes the next function signature from the build queue and returns it
    ///
    /// Returns:
    /// The function signature removed from the queue
    ///
    pub fn pop_build_queue(&mut self) -> Option<Func> {
        self.fn_build_queue.pop()
    }

    ///
    /// Gets the FuncSignature required to build a given FnCall
    /// This function automagically resolves interface params to match the type in the call
    /// If the returned signature has not been registered it will be added to the build queue
    /// The caller is always the value of the parents in the FnCall's ExpressionChain, or None if
    /// the FnCall's ExpressionChain does not have any parents
    /// TODO: Support for named_params to have an interface type
    ///
    /// Params:
    /// * `call` - The FnCall to get the FuncSignature of
    /// * `caller_opt` - The caller of this function, if any
    pub fn get_signature(
        &mut self,
        call: FnCall,
        caller_opt: Option<CrabValueType>,
        unnamed_values: &[CrabValueType],
        named_values: &HashMap<Ident, CrabValueType>,
    ) -> Result<FuncSignature> {
        // Get the source signature
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
        let source_fn = self
            .fn_sources
            .get(&mangled_name)
            .ok_or(CompileError::CouldNotFindFunction(call.name))?;

        self.verify_values(
            &source_fn.signature,
            unnamed_values,
            named_values,
            caller_opt,
        )?;

        // Create a map of params to their actual type
        let mut param_type_map = HashMap::new();
        for i in 0..unnamed_values.len() {
            let unnamed_arg = &unnamed_values[i];
            let unnamed_param = source_fn.signature.unnamed_params.get(i).unwrap();
            param_type_map.insert(unnamed_param.name.clone(), unnamed_arg.get_crab_type());
        }

        // Perform any interface mangling
        let interface_params: Vec<FnParam> = source_fn
            .signature
            .unnamed_params
            .clone()
            .into_iter()
            // .chain(
            //     source_sig
            //         .named_params
            //         .clone()
            //         .into_iter()
            //         .map(|named| FnParam::from(named))
            // )
            .filter(|param| match &param.crab_type {
                CrabType::STRUCT(id) => {
                    // Have to unwrap here, because rust doesn't let me return an error from filter
                    let ty = self.types.get_type(id).unwrap();
                    matches!(ty, ManagedType::INTERFACE(_))
                }
                CrabType::STRUCT_ARRAY(id, _) => {
                    // Have to unwrap here, because rust doesn't let me return an error from filter
                    let ty = self.types.get_type(id).unwrap();
                    matches!(ty, ManagedType::INTERFACE(_))
                }
                _ => false,
            })
            .map(|param| match &param.crab_type {
                CrabType::STRUCT(id) => {
                    let ty = self.types.get_type(id).unwrap();
                    if let ManagedType::INTERFACE(_) = ty {
                        param
                            .clone()
                            .with_type(param_type_map.get(&param.name).unwrap().clone())
                    } else {
                        unreachable!()
                    }
                }
                CrabType::STRUCT_ARRAY(id, _) => {
                    let ty = self.types.get_type(id).unwrap();
                    if let ManagedType::INTERFACE(_) = ty {
                        param
                            .clone()
                            .with_type(param_type_map.get(&param.name).unwrap().clone())
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!(),
            })
            .collect();
        let fully_mangled_name = add_param_mangles(&mangled_name, &interface_params);

        // Build proper Signature
        let mut unnamed_params = vec![];
        for i in 0..unnamed_values.len() {
            let unnamed_arg = &unnamed_values[i];
            // Probably shouldn't assume we have the correct number of unnamed params, but I'm lazy
            let unnamed_param = source_fn.signature.unnamed_params.get(i).unwrap();
            unnamed_params.push(unnamed_param.clone().with_type(unnamed_arg.get_crab_type()));
        }
        let generated_signature = FuncSignature {
            unnamed_params,
            name: fully_mangled_name.clone(),
            ..source_fn.signature.clone()
        };

        // Register if needed
        if !self.registered_fns.contains(&fully_mangled_name) {
            trace!("Registering new function {}", fully_mangled_name);
            self.fn_build_queue.push(Func {
                body: source_fn.body.clone(),
                signature: generated_signature.clone(),
            });
            self.register_function(generated_signature.clone(), false, None)?;
        }

        Ok(generated_signature)
    }

    ///
    /// Ensure that the given FuncSignature was called with valid args
    ///
    /// Params:
    /// * `signature` - The source signature of the called function
    /// * `unnamed_values` - The positional args the function is called with
    /// * `named_values` - The named args the function is called with
    /// * `caller_opt` - The caller of the function, if any
    ///
    /// Returns:
    /// An error if the function was called with invalid args
    ///
    fn verify_values(
        &self,
        signature: &FuncSignature,
        unnamed_values: &[CrabValueType],
        named_values: &HashMap<Ident, CrabValueType>,
        caller_opt: Option<CrabValueType>,
    ) -> Result<()> {
        // let unnamed_values_len = match caller_opt {
        //     Some(_) => unnamed_values.len()+1,
        //     None => unnamed_values.len(),
        // };
        let unnamed_values_len = unnamed_values.len();
        if unnamed_values_len != signature.unnamed_params.len() {
            trace!("****\n{:#?}\n****{:#?}\n", unnamed_values, signature);
            return Err(CompileError::PositionalArgumentCount(
                signature.name.clone(),
                signature.unnamed_params.len(),
                unnamed_values_len,
            ));
        }

        if let Some(val) = caller_opt {
            let unnamed_param = signature.unnamed_params.get(0).unwrap();
            if val.try_get_struct_name()? != unnamed_param.crab_type.try_get_struct_name()? {
                return Err(CompileError::ArgumentType(
                    signature.name.clone(),
                    unnamed_param.name.clone(),
                    unnamed_param.crab_type.clone(),
                    val.get_crab_type(),
                ));
            }
        }
        for j in 0..unnamed_values.len() {
            let val_type = unnamed_values.get(j).unwrap().get_crab_type();
            let unnamed_param = signature.unnamed_params.get(j).unwrap();

            if !self
                .types
                .is_a(val_type.clone(), unnamed_param.crab_type.clone())
            {
                return Err(CompileError::ArgumentType(
                    signature.name.clone(),
                    unnamed_param.name.clone(),
                    unnamed_param.crab_type.clone(),
                    val_type,
                ));
            }
        }

        let mut found_names = HashSet::new();
        for named_param in &signature.named_params {
            match named_values.get(&named_param.name) {
                Some(val) => {
                    found_names.insert(named_param.name.clone());
                    if !self
                        .types
                        .is_a(val.get_crab_type(), named_param.crab_type.clone())
                    {
                        return Err(CompileError::ArgumentType(
                            signature.name.clone(),
                            named_param.name.clone(),
                            named_param.crab_type.clone(),
                            val.get_crab_type(),
                        ));
                    }
                }
                None => {} // Do nothing
            }
        }

        if found_names.len() != named_values.len() {
            return Err(CompileError::InvalidNamedArgument(signature.name.clone()));
        }

        Ok(())
    }

    ///
    /// Register a builtin function
    /// Assumes that the function has already been built
    ///
    /// Params:
    /// * `signature` - The signature of the builtin function to register
    ///
    pub fn register_builtin(
        &mut self,
        signature: FuncSignature,
        variadic: bool,
        linkage: Option<Linkage>,
    ) -> Result<()> {
        let func = Func {
            signature: signature.clone(),
            body: CodeBlock { statements: vec![] },
        };
        self.register_function(signature, variadic, linkage)?;

        self.add_source(func);

        Ok(())
    }

    ///
    /// Register the function with the llvm stuff
    /// TODO: The linkage, mason! What does it mean?
    ///
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
        self.registered_fns.insert(func.name.clone());
        let fn_type =
            func.return_type
                .try_as_fn_type(self.context, self.module, &params, variadic)?;
        let _fn_value = self.module.add_function(&func.name, fn_type, linkage);
        Ok(())
    }
}
