use std::collections::HashMap;
use crate::parse::ast::{Ident, Struct};
use crate::compile::{Result, CompileError};

#[derive(Clone)]
pub struct StructManager {
    structs: HashMap<Ident, Struct>
}

impl StructManager {
    pub fn new() -> StructManager {
        Self { structs: HashMap::new() }
    }

    pub fn insert(&mut self, name: Ident, strct: Struct) -> Result<()> {
        return if self.structs.insert(name.clone(), strct).is_some() {
            Err(CompileError::StructRedefinition(name))
        } else {
            Ok(())
        }
    }

    pub fn get(&mut self, name: &Ident) -> Result<&Struct> {
        return self.structs.get(name).ok_or(CompileError::StructDoesNotExist(name.clone()))
    }
}
