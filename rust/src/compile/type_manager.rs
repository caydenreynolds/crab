use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabInterface, CrabType, Ident, Struct, StructIntr};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ManagedType {
    STRUCT(Struct),
    INTERFACE(CrabInterface),
}

#[derive(Debug, Clone, Default)]
pub struct TypeManager {
    /// All of the known types
    types: HashMap<Ident, ManagedType>,

    /// All of interfaces each struct implements
    intrs: HashMap<Ident, Vec<Ident>>,
}

impl TypeManager {
    pub fn new() -> TypeManager {
        Self::default()
    }

    ///
    /// Add a Struct directly from the CrabAst
    /// A struct that has not been added is not considered a valid type
    /// It is recommend to call add_struct on every Struct in the AST
    ///
    /// Params:
    /// * `strct` - The Struct to add to this TypeManager's structs
    ///
    pub fn register_struct(&mut self, strct: Struct) -> Result<()> {
        return if self
            .types
            .insert(strct.name.clone(), ManagedType::STRUCT(strct.clone()))
            .is_some()
        {
            Err(CompileError::StructRedefinition(strct.name))
        } else {
            Ok(())
        };
    }

    ///
    /// Add a StructIntr directly from the CrabAst
    /// StructIntrs are used to correctly identify is-a relationships
    /// It is recommend to call add_intr on every StructIntr in the AST
    /// This will error if the struct or any of it's interfaces haven't been registered
    ///
    /// Params:
    /// * `intr` - The Struct to add to this TypeManager's intrs
    ///
    pub fn register_intr(&mut self, intr: StructIntr) -> Result<()> {
        if let ManagedType::INTERFACE(_) = self
            .types
            .get(&intr.struct_name)
            .ok_or(CompileError::StructDoesNotExist(intr.struct_name.clone()))?
        {
            return Err(CompileError::NotAStruct);
        }
        for intfc in &intr.inters {
            if let ManagedType::STRUCT(_) = self
                .types
                .get(intfc)
                .ok_or(CompileError::StructDoesNotExist(intfc.clone()))?
            {
                return Err(CompileError::NotAnInterface);
            }
        }
        self.intrs.insert(intr.struct_name, intr.inters);

        Ok(())
    }

    ///
    /// Add a CrabInterface directly from the CrabAst
    /// An interface that has not been added is not considered a valid type
    /// It is recommend to call add_interface on every CrabInterface in the AST
    ///
    /// Params:
    /// * `intfc` - The Struct to add to this TypeManager's structs
    ///
    pub fn register_interface(&mut self, intfc: CrabInterface) -> Result<()> {
        return if self
            .types
            .insert(intfc.name.clone(), ManagedType::INTERFACE(intfc.clone()))
            .is_some()
        {
            Err(CompileError::InterfaceRedefinition(intfc.name))
        } else {
            Ok(())
        };
    }

    ///
    /// Try to get a type by name
    ///
    /// Params:
    /// * `name` - The name of the type to get
    ///
    /// Returns:
    /// The ManagedType with the matching name, may be either a struct or an interface
    pub fn get_type(&self, name: &Ident) -> Result<&ManagedType> {
        return self
            .types
            .get(name)
            .ok_or(CompileError::TypeDoesNotExist(name.clone()));
    }

    ///
    /// Returns whether lhs has an is-a relationship with rhs
    /// This returns true if lhs==rhs or lhs implements rhs
    /// This returns false otherwise
    ///
    /// Params:
    /// * `lhs` -
    /// * `rhs` -
    ///
    /// Returns:
    /// True if lhs is-a rhs, or false otherwise
    ///
    pub fn is_a(&self, lhs: CrabType, rhs: CrabType) -> bool {
        if lhs == rhs {
            return true;
        } else {
            let lhs_name = match lhs.try_get_struct_name() {
                Ok(id) => id,
                Err(_) => return false,
            };
            let rhs_name = match rhs.try_get_struct_name() {
                Ok(id) => id,
                Err(_) => return false,
            };

            match self.intrs.get(&lhs_name) {
                Some(intrs) => {
                    for intr in intrs {
                        if *intr == rhs_name {
                            return true;
                        }
                    }
                    return false;
                }
                None => return false,
            }
        }
    }
}
