use crate::compile::{CompileError, Result, TypeManager};
use crate::parse::ast::{CrabType, FnCall, FnParam, Func, FuncSignature, Ident, NamedFnParam};
use crate::quill::{PolyQuillType, QuillPointerType, QuillStructType, QuillValue};
use crate::util::{add_param_mangles, main_func_name, mangle_function_name, ListFunctional};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(super) struct FnManager {
    /// All fns that have been defined in the source code
    fn_sources: HashMap<Ident, Func>,

    /// All fns that have been registered to be built
    registered_fns: HashMap<Ident, Func>,

    /// All registered fns that have not been built yet
    fn_build_queue: Vec<Ident>,

    /// All of the registered types. Required for resolving params
    types: Rc<RefCell<TypeManager>>,
}

impl FnManager {
    pub fn new(types: Rc<RefCell<TypeManager>>) -> Self {
        Self {
            fn_sources: HashMap::new(),
            registered_fns: HashMap::new(),
            fn_build_queue: vec![],
            types,
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
        let main_name = mangle_function_name(&main_func_name(), None);

        // Verify there is a main fn
        let main = self
            .fn_sources
            .get(&main_name)
            .ok_or(CompileError::NoMain)?;

        // Do the thing
        self.fn_build_queue.push(main_name.clone());
        self.registered_fns.insert(main_name, main.clone());
        Ok(())
    }

    ///
    /// Removes the next function signature from the build queue and returns it
    ///
    /// Returns:
    /// The function signature removed from the queue
    ///
    pub fn pop_build_queue(&mut self) -> Option<Func> {
        let name = self.fn_build_queue.pop();
        name.map(|name| self.registered_fns.get(&name).expect("FnManager's build queue included a function name that isn't in the function sources").clone())
    }

    ///
    /// Returns whether or not the build queue is empty
    ///
    /// Returns:
    /// True if the build queue is empty, or false otherwise
    ///
    pub fn build_queue_empty(&self) -> bool {
        self.fn_build_queue.is_empty()
    }

    ///
    /// Retrieve a copy of a function's signature from the registered functions
    ///
    /// Params:
    /// * `name` - The name of the signature to get
    ///
    /// Returns:
    /// A copy of the requested signature
    ///
    pub fn get_source_signature(&self, name: &Ident) -> Result<FuncSignature> {
        Ok(self
            .fn_sources
            .get(name)
            .ok_or(CompileError::CouldNotFindFunction(name.clone()))?
            .signature
            .clone())
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
    ///
    pub fn get_signature(
        &mut self,
        call: &FnCall,
        caller_opt: &Option<QuillValue<PolyQuillType>>,
        unnamed_values: &[QuillValue<PolyQuillType>],
        named_values: &HashMap<Ident, QuillValue<PolyQuillType>>,
    ) -> Result<FuncSignature> {
        // For now, we're assuming we can only get a pointer to a struct value
        let caller_opt_t = match caller_opt {
            Some(caller) => Some(QuillStructType::try_from(
                QuillValue::<QuillPointerType>::try_from(caller.clone())?
                    .get_type()
                    .get_inner_type(),
            )?),
            None => None,
        };

        // Get the source signature
        let mangled_name = mangle_function_name(
            &call.name,
            caller_opt_t.map(|caller_t| caller_t.get_name()).as_ref(),
        );
        let source_fn = self
            .fn_sources
            .get(&mangled_name)
            .ok_or(CompileError::CouldNotFindFunction(call.name.clone()))?;

        self.verify_values(&source_fn.signature, unnamed_values, named_values)?;

        // Add all of the param names and types to a vec
        let unnamed_params = unnamed_values
            .iter()
            .zip(source_fn.signature.unnamed_params.iter())
            .try_fold(vec![], |params, (value, param)| {
                Result::Ok(
                    params.fpush(FnParam {
                        name: param.name.clone(),
                        crab_type: CrabType::STRUCT(
                            QuillStructType::try_from(
                                QuillValue::<QuillPointerType>::try_from(value.clone())?
                                    .get_type()
                                    .get_inner_type(),
                            )?
                            .get_name(),
                        ),
                    }),
                )
            })?;
        let named_params =
            source_fn
                .signature
                .named_params
                .iter()
                .try_fold(vec![], |params, param| {
                    let named_value = named_values.get(&param.name).unwrap();
                    Result::Ok(
                        params.fpush(NamedFnParam {
                            name: param.name.clone(),
                            crab_type: CrabType::STRUCT(
                                QuillStructType::try_from(
                                    QuillValue::<QuillPointerType>::try_from(named_value.clone())?
                                        .get_type()
                                        .get_inner_type(),
                                )?
                                .get_name(),
                            ),
                            expr: param.expr.clone(),
                        }),
                    )
                })?;

        // Just add all of the params to the mangle
        // As long as llvm doesn't enforce a maximum function name length, this should be fine
        let all_params = named_params
            .iter()
            .fold(unnamed_params.clone(), |params, named| {
                params.fpush(FnParam {
                    crab_type: named.crab_type.clone(),
                    name: named.name.clone(),
                })
            });
        let fully_mangled_name = add_param_mangles(&mangled_name, &all_params);

        // Build proper Signature
        let generated_signature = FuncSignature {
            unnamed_params,
            named_params,
            name: fully_mangled_name.clone(),
            return_type: source_fn.signature.return_type.clone(),
        };

        // Register if needed
        if !self.registered_fns.contains_key(&fully_mangled_name) {
            self.registered_fns.insert(
                fully_mangled_name.clone(),
                Func {
                    body: source_fn.body.clone(),
                    signature: generated_signature.clone(),
                },
            );
            self.fn_build_queue.push(fully_mangled_name);
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
        unnamed_values: &[QuillValue<PolyQuillType>],
        named_values: &HashMap<Ident, QuillValue<PolyQuillType>>,
    ) -> Result<()> {
        if unnamed_values.len() != signature.unnamed_params.len() {
            return Err(CompileError::PositionalArgumentCount(
                signature.name.clone(),
                signature.unnamed_params.len(),
                unnamed_values.len(),
            ));
        }

        unnamed_values
            .iter()
            .zip(signature.unnamed_params.iter())
            .try_for_each(|(value, param)| {
                let val_t = CrabType::STRUCT(
                    QuillStructType::try_from(
                        QuillValue::<QuillPointerType>::try_from(value.clone())?
                            .get_type()
                            .get_inner_type(),
                    )?
                    .get_name(),
                );
                match self.types.borrow().is_a(&val_t, &param.crab_type) {
                    true => Ok(()),
                    false => Err(CompileError::ArgumentType(
                        signature.name.clone(),
                        param.name.clone(),
                        param.crab_type.clone(),
                        val_t,
                    )),
                }
            })?;
        signature.named_params.iter().try_for_each(|param| {
            let named_value = named_values
                .get(&param.name)
                .ok_or(CompileError::ArgumentNotSupplied(param.name.clone()))?;
            let val_t = CrabType::STRUCT(
                QuillStructType::try_from(
                    QuillValue::<QuillPointerType>::try_from(named_value.clone())?
                        .get_type()
                        .get_inner_type(),
                )?
                .get_name(),
            );
            match self.types.borrow().is_a(&val_t, &param.crab_type) {
                true => Ok(()),
                false => Err(CompileError::ArgumentType(
                    signature.name.clone(),
                    param.name.clone(),
                    param.crab_type.clone(),
                    val_t,
                )),
            }
        })?;

        Ok(())
    }
}
