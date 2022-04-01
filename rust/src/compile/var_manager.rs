use crate::compile::{CompileError, CrabValueType, Result};
use crate::parse::ast::Ident;
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct VarManager<'ctx> {
    vars: HashMap<Ident, CrabValueType<'ctx>>,
}

impl<'ctx> VarManager<'ctx> {
    pub fn new() -> VarManager<'ctx> {
        Self::default()
    }

    pub fn assign(&mut self, name: Ident, value: CrabValueType<'ctx>) -> Result<()> {
        return if self.vars.insert(name.clone(), value).is_some() {
            Err(CompileError::VarAlreadyExists(name))
        } else {
            Ok(())
        };
    }

    pub fn reassign(&mut self, name: Ident, value: CrabValueType<'ctx>) -> Result<()> {
        return if self.vars.insert(name.clone(), value).is_none() {
            Err(CompileError::VarDoesNotExist(name))
        } else {
            Ok(())
        };
    }

    pub fn get(&mut self, name: &Ident) -> Result<CrabValueType<'ctx>> {
        return Ok(self
            .vars
            .get(name)
            .ok_or(CompileError::VarDoesNotExist(name.clone()))?
            .clone());
    }
}
