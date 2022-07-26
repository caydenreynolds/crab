use crate::compile::{CompileError, CrabValue, Result, TypeManager};
use crate::parse::ast::{CrabType, FnCall, Func, FuncSignature, Ident, NamedParam, PosParam, StructId};
use crate::util::{ListFunctional, MapFunctional, magic_main_func_name};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;
use std::default::Default;

#[derive(Debug, Clone)]
pub(super) struct FnManager {
    /// All fns that have been defined outside of impl blocks
    fn_sources: HashMap<Ident, Func>,

    /// All the fns that have been defined inside impl blocks
    impl_sources: HashMap<ImplFuncId, Func>,

    /// All fns that have been registered to be built
    registered_fns: HashSet<FuncSignature>,

    /// All registered fns that have not been built yet
    fn_build_queue: Vec<Func>,

    /// All of the registered types. Required for resolving params
    types: Rc<RefCell<TypeManager>>,
}

impl FnManager {
    pub fn new(types: Rc<RefCell<TypeManager>>) -> Self {
        Self {
            types,
            fn_sources: Default::default(),
            impl_sources: Default::default(),
            registered_fns: Default::default(),
            fn_build_queue: Default::default(),
        }
    }

    ///
    /// Add a Func directly from the CrabAst
    /// A function that has not been added as source cannot be registered or built
    /// The source func will be added regardless of whether or not it belongs to an impl
    ///
    /// Params:
    /// * `source` - The Func to add to this FnManager's sources
    ///
    pub fn add_source(&mut self, source: Func) {
        match &source.signature.caller_id {
            None => {
                self.fn_sources
                    .insert(source.signature.name.clone(), source);
            },
            Some(si) => {
                self.impl_sources.insert(
                    ImplFuncId::from_structid(source.signature.name.clone(), si),source
                );
            },
        };
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
            .get(&magic_main_func_name())
            .ok_or(CompileError::NoMain)?;
        self.fn_build_queue.push(main.clone());
        self.registered_fns.insert(main.signature.clone());
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
    pub fn get_source_signature(&self, name: &Ident, caller_opt: Option<CrabType>) -> Result<FuncSignature> {
        Ok(self.get_source(&name, caller_opt)?.signature)
    }

    ///
    /// Gets the FuncSignature required to build a given FnCall
    /// This function automagically resolves interface params to match the type in the call
    /// If the returned signature has not been registered it will be added to the build queue
    /// The caller is always the value of the parents in the FnCall's ExpressionChain, or None if
    /// the FnCall's ExpressionChain does not have any parents
    ///
    /// Params:
    /// * `call` - The FnCall to get the FuncSignature of
    /// * `caller_opt` - The caller of this function, if any
    ///
    pub fn get_signature(
        &mut self,
        call: &FnCall,
        caller_opt: Option<CrabType>,
        pos_values: &[CrabValue],
        named_values: &BTreeMap<Ident, CrabValue>,
    ) -> Result<FuncSignature> {

        let source_fn = self.get_source(&call.name, caller_opt.clone())?;

        let pos_params = match caller_opt {
            None => vec![],
            Some(caller) => {
                vec![PosParam { name: String::from("self"), crab_type: caller.clone() }]
            }
        };
        let pos_params = pos_values
            .iter()
            .zip(source_fn.signature.pos_params.iter())
            .try_fold(pos_params, |pos_params, (value, param)| {
                match self.types.borrow().is_a(&value.crab_type, &param.crab_type) {
                    true => {
                        Result::Ok(
                            pos_params.fpush(PosParam {
                                name: param.name.clone(),
                                crab_type: value.crab_type.clone(),
                            })
                        )
                    },
                    false => {
                        Result::Err(CompileError::ArgumentType(
                            call.name.clone(),
                            param.name.clone(),
                            param.crab_type.clone(),
                            value.crab_type.clone(),
                        ))
                    }
                }
            })?;
        let named_params = named_values
            .iter()
            .zip(source_fn.signature.named_params.iter())
            .try_fold(BTreeMap::new(), |named_params, ((_, arg), (_, param))| {
                match self.types.borrow().is_a(&arg.crab_type, &param.crab_type) {
                    true => {
                        Result::Ok(
                            named_params.finsert(
                                param.name.clone(),
                                NamedParam {
                                    name: param.name.clone(),
                                    crab_type: arg.crab_type.clone(),
                                    expr: param.expr.clone(),
                                },
                            )
                        )
                    },
                    false => {
                        Result::Err(CompileError::ArgumentType(
                            call.name.clone(),
                            param.name.clone(),
                            param.crab_type.clone(),
                            arg.crab_type.clone(),
                        ))
                    }
                }
            })?;

        // Build proper Signature
        let generated_signature = FuncSignature {
            pos_params,
            named_params,
            name: source_fn.signature.name.clone(),
            return_type: source_fn.signature.return_type.clone(),
            caller_id: None,
        };

        // Always register, only add to build_queue if this func wasn't already registered
        if self.registered_fns.insert(generated_signature.clone()) {
            self.fn_build_queue.push(Func {
                body: source_fn.body.clone(),
                signature: generated_signature.clone()
            });
        }

        Ok(generated_signature)
    }

    fn get_source(&self, name: &Ident, caller_opt: Option<CrabType>) -> Result<Func> {
        let func_opt = match caller_opt {
            Some(caller) => self
                .impl_sources
                .get(&ImplFuncId::from_crabtype(name.clone(), &caller)?),
            None => self
                .fn_sources
                .get(name),
        };
        func_opt.ok_or(CompileError::CouldNotFindFunction(name.clone())).cloned()
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct ImplFuncId {
    func_name: Ident,
    struct_name: Ident,
}

impl ImplFuncId {
    fn from_crabtype(func_name: Ident, ct: &CrabType) -> Result<Self> {
        let struct_name = match ct {
            CrabType::VOID | CrabType::PRIM_INT | CrabType::PRIM_STR | CrabType::PRIM_BOOL => {
                Err(CompileError::NotAStruct(
                    StructId::from_name(format!("{}", ct)),
                    String::from("ImplFuncId::new()")
                ))
            },
            CrabType::SIMPLE(name) | CrabType::TMPL(name, _) => {
                Ok(name.clone())
            },
            CrabType::LIST(ct ) => {
                Ok(ct.try_get_struct_name()?.clone())
            },
        }?;
        Ok(Self {
            func_name,
            struct_name,
        })
    }

    fn from_structid(func_name: Ident, si: &StructId) -> Self {
        Self {
            func_name,
            struct_name: si.name.clone(),
        }
    }
}
