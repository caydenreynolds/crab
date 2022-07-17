use crate::compile::builtins::get_builtin_strct_definition;
use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabInterface, CrabType, FuncSignature, Ident, StrctBodyType, CrabStruct, StructIntr, StructIdent};
use crate::quill::{
    PolyQuillType, QuillBoolType, QuillFloatType, QuillFnType, QuillIntType, QuillListType,
    QuillPointerType, QuillStructType, QuillVoidType,
};
use crate::util::{extract_struct_name, ListFunctional};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(super) struct StructField {
    pub name: Ident,
    pub field_type: PolyQuillType,
}

#[derive(Debug, Clone)]
pub(super) struct Struct {
    pub tmpl_names: Vec<Ident>,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub(super) enum ManagedType {
    STRUCT(Struct),
    INTERFACE(CrabInterface),
}

///
/// Struct that's responsible for converting from crab types to quill types
/// This includes structs, interfaces, and primitive types
///
#[derive(Debug, Default)]
pub(super) struct TypeManager {
    /// All of the types that have been registered, but may or may not have been used. Includes structs and interfaces.
    registered_types: HashMap<StructIdent, ManagedType>,

    /// All of the structs that have been used and must therefore be added to the quill
    included_types: HashSet<StructIdent>,

    /// All of interfaces each struct implements
    intrs: HashMap<StructIdent, Vec<StructIdent>>,
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
        let my_strct = match strct.body {
            StrctBodyType::COMPILER_PROVIDED => get_builtin_strct_definition(&strct.name)?,
            StrctBodyType::FIELDS(fields) => {
                Struct {
                    tmpl_names: strct.tmpls,
                    fields: fields.into_iter().map(|field| {
                        let pqt = match field.crab_type {
                            CrabType::STRUCT(name) => QuillStructType::new(name).into(),
                            _ => self.get_quill_type(&field.crab_type).unwrap(),
                        };
                        StructField {
                            name: field.name,
                            field_type: pqt
                        }
                    }).collect()
                }
            }
        };
        return if self
            .registered_types
            .insert(strct.id.clone(), ManagedType::STRUCT(my_strct))
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
        let interface_id = intfc.id.clone();
        return if self
            .registered_types
            .insert(intfc.id.clone(), ManagedType::INTERFACE(intfc))
            .is_some()
        {
            Err(CompileError::InterfaceRedefinition(interface_id))
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
    pub fn get_type(&mut self, id: &StructIdent) -> Result<&ManagedType> {
        let mt = self
            .registered_types
            .get(&id)
            .ok_or(CompileError::TypeDoesNotExist(id.clone()))?;
        match &mt {
            ManagedType::STRUCT(_) => {
                self.included_types.insert(name.clone());
            }
            ManagedType::INTERFACE(_) => {} // Do nothing
        }
        Ok(mt)
    }

    // ///
    // /// Try to get a struct by name
    // ///
    // /// Params:
    // /// * `name` - The name of the struct to get
    // ///
    // /// Returns:
    // /// The struct with the matching name, or an error if there is no struct with the matching name
    // ///
    // pub fn get_struct(&mut self, name: &Ident) -> Result<&Struct> {
    //     match self.get_type(name)? {
    //         ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
    //             name.clone(),
    //             String::from("TypeManager::get_struct"),
    //         )),
    //         ManagedType::STRUCT(strct) => Ok(strct),
    //     }
    // }

    // ///
    // /// Get the given crab type as a quill type
    // /// If a struct type is returned from this function, it will be added to the `included_types` map
    // /// this will register that struct to be included in the Quill
    // ///
    // /// Params:
    // /// * `ct` - The CrabType to get as a quill type
    // ///
    // /// Returns:
    // /// A PolyQuillType that is equivalent to the given CrabType
    // ///
    // pub fn get_quill_type(&mut self, ct: &CrabType) -> Result<PolyQuillType> {
    //     Ok(match ct {
    //         CrabType::UINT8 => QuillIntType::new(8).into(),
    //         CrabType::UINT64 => QuillIntType::new(64).into(),
    //         CrabType::STRING => unimplemented!(),
    //         CrabType::VOID => QuillVoidType::new().into(),
    //         CrabType::FLOAT => QuillFloatType::new().into(),
    //         CrabType::BOOL => QuillBoolType::new().into(),
    //         CrabType::STRUCT(name) => match self.get_type(name)?.clone() {
    //             ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
    //                 name.clone(),
    //                 String::from("TypeManager::get_quill_type"),
    //             ))?,
    //             ManagedType::STRUCT(_) => {
    //                 QuillPointerType::new(QuillStructType::new(name.clone())).into()
    //             }
    //         },
    //         //TODO: This len should be dynamic, but I don't think we ever actually use list types atm
    //         CrabType::LIST(t) => {
    //             QuillListType::new_const_length(self.get_quill_type(t)?, 1000).into()
    //         }
    //     })
    // }

    // ///
    // /// Get a quill struct from the crab struct that has the given name
    // /// If a struct type is returned from this function, it will be added to the `included_types` map
    // /// this will register that struct to be included in the Quill
    // ///
    // /// Params:
    // /// * `name` - The name of the struct to get
    // ///
    // /// Returns:
    // /// A QuillStructType that has the given name
    // ///
    // pub fn get_quill_struct(&mut self, id: &Ident) -> Result<QuillStructType> {
    //     match self.get_type(id)?.clone() {
    //         ManagedType::INTERFACE(_) => Err(CompileError::NotAStruct(
    //             id.clone(),
    //             String::from("TypeManager::get_quill_struct"),
    //         )),
    //         ManagedType::STRUCT(_) => Ok(QuillStructType::new(id.clone())),
    //     }
    // }

    // ///
    // /// Get the given crab type as a quill function type type
    // /// If a struct type is returned from this function, it will be added to the `included_types` map
    // /// this will register that struct to be included in the Quill
    // ///
    // /// Params:
    // /// * `fs` - The signature to get as a quill type
    // ///
    // /// Returns:
    // /// A QuillFnType that is equivalent to the given FuncSignature
    // ///
    // pub fn get_quill_fn_type(&mut self, fs: FuncSignature) -> Result<QuillFnType> {
    //     let params = fs
    //         .unnamed_params
    //         .into_iter()
    //         .try_fold(vec![], |params, up| {
    //             Result::Ok(params.fpush((up.name, self.get_quill_type(&up.crab_type)?)))
    //         })?;
    //     let params = fs.named_params.into_iter().try_fold(params, |params, np| {
    //         Result::Ok(params.fpush((np.name, self.get_quill_type(&np.crab_type)?)))
    //     })?;
    //     let ret_t = match self.get_quill_type(&fs.return_type)? {
    //         PolyQuillType::VoidType(_) => None,
    //         t => Some(t),
    //     };
    //     Ok(QuillFnType::new(ret_t, params))
    // }

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
    pub fn is_a(&self, lhs: &StructIdent, rhs: &StructIdent) -> bool {
        if lhs == rhs {
            return true;
        } else {
            match self.intrs.get(&lhs) {
                Some(intrs) => {
                    for intr in intrs {
                        if *intr == rhs {
                            return true;
                        }
                    }
                    return false;
                }
                None => return false,
            }
        }
    }

    pub fn get_included_type_names(&self) -> &HashSet<StructIdent> {
        &self.included_types
    }
}
