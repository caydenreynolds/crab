use crate::compile::builtins::get_builtin_strct_definition;
use crate::compile::{CompileError, Result};
use crate::parse::ast::{
    CrabInterface, CrabStruct, CrabType, FuncSignature, Ident, StructBody, StructId, StructIntr,
};
use crate::quill::{
    PolyQuillType, QuillFnType, QuillListType, QuillPointerType, QuillStructType, QuillVoidType,
};
use crate::util::{ListFunctional, MapFunctional};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(super) enum ManagedType {
    STRUCT(CrabStruct),
    INTERFACE(CrabInterface),
}

impl ManagedType {
    fn as_struct(&self) -> Result<&CrabStruct> {
        match self {
            ManagedType::STRUCT(s) => Ok(s),
            ManagedType::INTERFACE(i) => Err(CompileError::NotAStruct(
                StructId::from_name(i.name.clone()),
                String::from("ManagedType::as_struct"),
            )),
        }
    }
}

///
/// Struct that's responsible for converting from crab types to quill types
/// This includes structs, interfaces, and primitive types
///
#[derive(Debug, Clone, Default)]
pub(super) struct TypeManager {
    /// All of the types that have been registered, but may or may not have been used
    /// Includes structs and interfaces
    /// Duplicate names are disallowed here
    registered_types: HashMap<Ident, ManagedType>,

    /// All of the structs that have been used and must therefore be added to the quill
    /// Duplicate names are allowed here
    /// Any StructId tmpls must be resolved to concrete types
    included_types: HashSet<CrabStruct>,

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
            .insert(strct.id.name.clone(), ManagedType::STRUCT(strct.clone()))
            .is_some()
        {
            Err(CompileError::StructRedefinition(strct.id.name))
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
            .get(&intr.struct_id.name)
            .ok_or(CompileError::StructDoesNotExist(intr.struct_id.clone()))?
        {
            return Err(CompileError::NotAStruct(
                intr.struct_id.clone(),
                String::from("TypeManager::register_intr"),
            ));
        }
        for intfc in &intr.inters {
            let si = StructId::from_name(intfc.clone());
            if let ManagedType::STRUCT(_) = self
                .registered_types
                .get(&si.name)
                .ok_or(CompileError::StructDoesNotExist(si.clone()))?
            {
                return Err(CompileError::NotAnInterface);
            }
        }
        self.intrs.insert(intr.struct_id.name, intr.inters);

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
    fn get_type(&mut self, ct: &CrabType) -> Result<ManagedType> {
        let (ct_name, ct_tmpls) = match ct {
            CrabType::SIMPLE(id) => (id.clone(), vec![]),
            CrabType::LIST(id) => (id.try_get_struct_name()?, vec![]),
            CrabType::TMPL(id, tmpls) => (id.clone(), tmpls.clone()),
            CrabType::VOID => return Err(CompileError::VoidType),
            _ => {
                return Err(CompileError::NotAStruct(
                    StructId::from_name(format!("{}", ct)),
                    String::from("TypeManager::get_type"),
                ))
            }
        };
        let mt = self
            .registered_types
            .get(&ct_name)
            .ok_or(CompileError::TypeDoesNotExist(ct_name.clone()))?
            .clone();
        let mt = match &mt {
            ManagedType::STRUCT(strct) => {
                // Check the ct_tmpls has valid types
                // Also add any types included in the CrabType to the list of registered types
                ct_tmpls
                    .iter()
                    .try_for_each(|ct| self.get_type(ct)?.as_struct().map(|_| ()))?;

                let resolved_struct = strct.clone().resolve(ct_tmpls.as_slice())?;
                self.included_types.insert(resolved_struct.clone());
                ManagedType::STRUCT(resolved_struct)
            }
            ManagedType::INTERFACE(_) => mt.clone(),
        };
        Ok(mt)
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
            CrabType::VOID => QuillVoidType::new().into(),
            CrabType::SIMPLE(_) | CrabType::TMPL(_, _) => {
                let name = self.get_type(ct)?.as_struct()?.id.mangle();
                QuillPointerType::new(QuillStructType::new(name)).into()
            }
            //TODO: This len should be dynamic
            CrabType::LIST(t) => {
                QuillListType::new_const_length(self.get_quill_type(t)?, 1000).into()
            }
            _ => unreachable!(),
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
    pub fn get_quill_struct(&mut self, id: &CrabType) -> Result<QuillStructType> {
        Ok(QuillStructType::new(
            self.get_type(id)?.as_struct()?.id.mangle(),
        ))
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
        let params = fs.pos_params.into_iter().try_fold(vec![], |params, up| {
            Result::Ok(params.fpush((up.name, self.get_quill_type(&up.crab_type)?)))
        })?;
        let params = fs
            .named_params
            .into_iter()
            .try_fold(params, |params, (_, np)| {
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

            if lhs_name == rhs_name {
                return true;
            }

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
    pub fn get_fields(&mut self, id: &CrabType) -> Result<HashMap<String, PolyQuillType>> {
        Ok(match self.get_type(id)?.as_struct()?.body.clone() {
            StructBody::COMPILER_PROVIDED => {
                get_builtin_strct_definition(&id.try_get_struct_name()?)?.clone()
            }
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

    ///
    /// Returns the CrabTypes of the fields of a given struct type
    /// If the struct type's fields are compiler provided, they will be fetched from the ast
    /// Otherwise, they will be resolved from this type manager's struct definitions
    ///
    pub fn get_field_types(&mut self, id: &CrabType) -> Result<HashMap<String, CrabType>> {
        Ok(match self.get_type(id)?.as_struct()?.body.clone() {
            StructBody::COMPILER_PROVIDED => todo!(),
            StructBody::FIELDS(fields) => {
                fields
                    .into_iter()
                    .try_fold(HashMap::new(), |fields, field| {
                        Result::Ok(fields.finsert(field.name.clone(), field.crab_type.clone()))
                    })?
            }
        })
    }

    pub fn get_included_type_names(&self) -> &HashSet<CrabStruct> {
        &self.included_types
    }
}
