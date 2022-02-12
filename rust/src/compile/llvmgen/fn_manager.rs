use crate::compile::{CompileError, Result};
use crate::parse::ast::{FuncSignature, Ident};
use std::collections::HashMap;

#[derive(Clone)]
pub struct FnManager {
    fns: HashMap<Ident, FuncSignature>,
}

impl FnManager {
    pub fn new() -> FnManager {
        Self {
            fns: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: Ident, fn_sig: FuncSignature) -> Result<()> {
        return if self.fns.insert(name.clone(), fn_sig).is_some() {
            Err(CompileError::FunctionRedefinition(name))
        } else {
            Ok(())
        };
    }

    pub fn get(&mut self, name: &Ident) -> Result<&FuncSignature> {
        return self
            .fns
            .get(name)
            .ok_or(CompileError::CouldNotFindFunction(name.clone()));
    }
}
