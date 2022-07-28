use crate::compile::{CompileError, CrabValue, Result};
use crate::parse::ast::Ident;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(super) struct VarManager(HashMap<Ident, CrabValue>);

impl VarManager {
    pub(super) fn new() -> Self {
        Self(HashMap::new())
    }

    ///
    /// Assigns a new value with a given name and value
    /// Returns an error if a variable already exists with the given name
    ///
    /// Params:
    /// * `name` - The name of the variable to assign
    /// * `value` - The value of the variable to assign
    ///
    pub(super) fn assign(&mut self, name: Ident, value: CrabValue) -> Result<()> {
        match self.0.insert(name.clone(), value) {
            Some(_) => Err(CompileError::VarAlreadyExists(name)),
            None => Ok(()),
        }
    }

    ///
    /// Reassigns a new value with a given name and value
    /// Returns an error if a variable does not already exist with the given name
    ///
    /// Params:
    /// * `name` - The name of the variable to assign
    /// * `value` - The value of the variable to assign
    ///
    pub(super) fn reassign(&mut self, name: Ident, value: CrabValue) -> Result<()> {
        match self.0.insert(name.clone(), value) {
            Some(_) => Ok(()),
            None => Err(CompileError::VarDoesNotExist(name)),
        }
    }

    ///
    /// Retrieve a value from the var manager by name
    ///
    /// Params:
    /// * `name` - The name of the variable to retrieve
    ///
    /// Returns:
    /// The QuillValue with the given name
    ///
    pub(super) fn get(&mut self, name: &Ident) -> Result<&CrabValue> {
        match self.0.get(name) {
            None => Err(CompileError::VarDoesNotExist(name.clone())),
            Some(val) => Ok(val),
        }
    }
}
