use crate::compile::builtins::get_builtin_strct_definition;
use crate::compile::{CompileError, Result};
use crate::parse::ast::{
    CrabInterface, CrabStruct, CrabType, FuncSignature, Ident, StructBody, StructIntr,
};
use crate::quill::{
    PolyQuillType, QuillBoolType, QuillFloatType, QuillFnType, QuillIntType, QuillListType,
    QuillPointerType, QuillStructType, QuillVoidType,
};
use crate::util::{ListFunctional, MapFunctional};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(super) enum ManagedType {
    STRUCT(CrabStruct),
    INTERFACE(CrabInterface),
}

///
/// Struct that's responsible for converting from crab types to quill types
/// This includes structs, interfaces, and primitive types
///
#[derive(Debug, Clone, Default)]
pub(super) struct TypeManager {
    /// All of the types that have been registered, but may or may not have been used. Includes structs and interfaces.
    registered_types: HashMap<Ident, ManagedType>,

    /// All of the structs that have been used and must therefore be added to the quill
    included_types: HashSet<Ident>,

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
    pub fn register_struct(&mut self, strct: CrabStruct) -> Result<()> {
        return if self
            .registered_types
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
            .registered_types
            .get(&intr.struct_name)
            .ok_or(CompileError::StructDoesNotExist(intr.struct_name.clone()))?
        {
            return Err(CompileError::NotAStruct(
                intr.struct_name.clone(),
                String::from("TypeManager::register_intr"),
            ));
        }
        for intfc in &intr.inters {
            if let ManagedType::STRUCT(_) = self
                .registered_types
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
            .registered_types
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
    /// If a struct type is returned from this function, it will be added to the `included_types` map
    /// this will register that struct to be included in the Quill
    ///
    /// Params:
    /// * `name` - The name of the type to get
    ///
    /// Returns:
    /// The ManagedType with the matching name, may be either a struct or an interface
    ///
    pub fn get_type(&mut self, name: &Ident) -> Result<&ManagedType> {
        let mt = self
            .registered_types
            .get(name)
            .ok_or(CompileError::TypeDoesNotExist(name.clone()))?;
        match &mt {
            ManagedType::STRUCT(_) => {
                self.included_types.insert(name.clone());
            }
            ManagedType::INTERFACE(_) => {} // Do nothing
        }
        Ok(mt)
    }

    ///
    /// Try to get a struct by name
    ///
    /// Params:
    /// * `name` - The name of the struct to get
    ///
    /// Returns:
    /// The struct with the matching name, or an error if there is no struct with the matching name
    ///
    pub fn get_struct(&mut self, name: &Ident) -> Result<&CrabStruct> {
        match self.get_type(name)? {
            ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
                name.clone(),
                String::from("TypeManager::get_struct"),
            )),
            ManagedType::STRUCT(strct) => Ok(strct),
        }
    }

    ///
    /// Get the given crab type as a quill type
    /// If a struct type is returned from this function, it will be added to the `included_types` map
    /// this will register that struct to be included in the Quill
    ///
    /// Params:
    /// * `ct` - The CrabType to get as a quill type
    ///
    /// Returns:
    /// A PolyQuillType that is equivalent to the given CrabType
    ///
    pub fn get_quill_type(&mut self, ct: &CrabType) -> Result<PolyQuillType> {
        Ok(match ct {
            CrabType::UINT8 => QuillIntType::new(8).into(),
            CrabType::UINT64 => QuillIntType::new(64).into(),
            CrabType::STRING => unimplemented!(),
            CrabType::VOID => QuillVoidType::new().into(),
            CrabType::FLOAT => QuillFloatType::new().into(),
            CrabType::BOOL => QuillBoolType::new().into(),
            CrabType::STRUCT(name) => match self.get_type(name)?.clone() {
                ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
                    name.clone(),
                    String::from("TypeManager::get_quill_type"),
                ))?,
                ManagedType::STRUCT(strct) => {
                    QuillPointerType::new(QuillStructType::new(strct.name)).into()
                }
            },
            //TODO: This len should be dynamic
            CrabType::LIST(t) => {
                QuillListType::new_const_length(self.get_quill_type(t)?, 1000).into()
            }
        })
    }

    ///
    /// Get a quill struct from the crab struct that has the given name
    /// If a struct type is returned from this function, it will be added to the `included_types` map
    /// this will register that struct to be included in the Quill
    ///
    /// Params:
    /// * `name` - The name of the struct to get
    ///
    /// Returns:
    /// A QuillStructType that has the given name
    ///
    pub fn get_quill_struct(&mut self, id: &Ident) -> Result<QuillStructType> {
        match self.get_type(id)?.clone() {
            ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
                id.clone(),
                String::from("TypeManager::get_quill_struct"),
            )),
            ManagedType::STRUCT(strct) => Ok(QuillStructType::new(strct.name)),
        }
    }

    ///
    /// Get the given crab type as a quill function type type
    /// If a struct type is returned from this function, it will be added to the `included_types` map
    /// this will register that struct to be included in the Quill
    ///
    /// Params:
    /// * `fs` - The signature to get as a quill type
    ///
    /// Returns:
    /// A QuillFnType that is equivalent to the given FuncSignature
    ///
    pub fn get_quill_fn_type(&mut self, fs: FuncSignature) -> Result<QuillFnType> {
        let params = fs
            .unnamed_params
            .into_iter()
            .try_fold(vec![], |params, up| {
                Result::Ok(params.fpush((up.name, self.get_quill_type(&up.crab_type)?)))
            })?;
        let params = fs.named_params.into_iter().try_fold(params, |params, np| {
            Result::Ok(params.fpush((np.name, self.get_quill_type(&np.crab_type)?)))
        })?;
        let ret_t = match self.get_quill_type(&fs.return_type)? {
            PolyQuillType::VoidType(_) => None,
            t => Some(t),
        };
        Ok(QuillFnType::new(ret_t, params))
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
    pub fn is_a(&self, lhs: &CrabType, rhs: &CrabType) -> bool {
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

    ///
    /// Returns the fields of a given struct type
    /// If the struct type's fields are compiler provided, they will be fetched from the ast
    /// Otherwise, they will be resolved from this type manager's struct definitions
    ///
    pub fn get_fields(&mut self, name: &Ident) -> Result<HashMap<String, PolyQuillType>> {
        Ok(match self.get_struct(name)?.body.clone() {
            StructBody::COMPILER_PROVIDED => get_builtin_strct_definition(&name)?.clone(),
            StructBody::FIELDS(fields) => {
                fields
                    .into_iter()
                    .try_fold(HashMap::new(), |fields, field| {
                        Result::Ok(
                            fields.finsert(
                                field.name.clone(),
                                self.get_quill_type(&field.crab_type)?,
                            ),
                        )
                    })?
            }
        })
    }

    pub fn get_included_type_names(&self) -> &HashSet<Ident> {
        &self.included_types
    }
}
