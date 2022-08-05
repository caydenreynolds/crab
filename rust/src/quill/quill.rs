use crate::quill::{FnNib, Nib, PolyQuillType, QuillError, QuillFnType, QuillType, Result};
use crate::util::{ListFunctional, ListReplace, MapFunctional};
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use log::{debug, error, trace};
use std::collections::HashMap;
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};

///
/// Essentially a combined inkwell context & module
///
#[derive(Debug, Default)]
pub struct Quill {
    functions: HashMap<String, (QuillFnType, FnNib)>,
    struct_types: HashMap<String, StructDefinition>,
    external_functions: HashMap<String, QuillFnType>,
}

impl Quill {
    pub fn new() -> Self {
        Self::default()
    }

    ///
    /// Commits all of the instructions contained in this quill to an intermediate artifact llvm understands
    ///
    /// Params:
    /// * `out_path` - Where to place the generated artifact. If no file extension is provided, one that matches the artifact_type will be chosen
    /// * `artifact_type` - The type of artifact to build
    ///
    pub fn commit(self, out_path: &Path, artifact_type: &ArtifactType, verify: bool) -> Result<()> {
        trace!("Called Quill::commit with out_path {:?}", out_path);

        if let ArtifactType::QIR = artifact_type {
            let out_path = out_path.with_extension("qir");
            debug!("Writing {:?}", out_path);
            fs::write(&out_path, format!("{:#?}", self))?;
            return Ok(());
        }

        // We have to create our module inside of the catch_unwind loop
        // Because a *reference* to a module is not UnwindSafe.
        // But, because the module borrows from context, the context must be created outside of the closure
        // so that we can have the closure return the module.
        // Somehow a reference to a context is UnwindSafe? I understand nothing.
        let context = Context::create();
        let result = catch_unwind(|| {
            let module = context.create_module("module");

            // First, a pass to tell llvm about all of the possible struct types
            debug!("Registering struct types");
            let l_sts = self.struct_types.iter().fold(vec![], |l_sts, (name, _)| {
                trace!("Registered struct type with name {}", name);
                l_sts.fpush(context.opaque_struct_type(name))
            });

            // Next, a pass to tell llvm the contents of the structs
            debug!("Registering struct definitions");
            self.struct_types
                .iter()
                .zip(l_sts.into_iter())
                .try_for_each(|((name, q_st), l_st)| {
                    trace!("Building body of struct {}", name);
                    let body = q_st
                        .get_types()
                        .iter()
                        .try_fold(vec![], |field_types, pqt| {
                            Result::Ok(field_types.fpush(pqt.as_llvm_type(&context, &module)?))
                        })?;
                    l_st.set_body(&body, false);
                    Result::Ok(())
                })?;

            // Register the external functions
            self.external_functions
                .iter()
                .try_for_each(|(name, header)| {
                    trace!("Registering external fn type {}", name);
                    let params =
                        header
                            .get_params()
                            .iter()
                            .try_fold(vec![], |params, (_, param)| {
                                Result::Ok(params.fpush(BasicMetadataTypeEnum::from(
                                    param.as_llvm_type(&context, &module)?,
                                )))
                            })?;
                    let fn_t = match header.get_ret_type() {
                        None => context.void_type().fn_type(&params, false),
                        Some(pqt) => pqt.as_llvm_type(&context, &module)?.fn_type(&params, false),
                    };
                    module.add_function(name, fn_t, Some(Linkage::External));
                    Result::Ok(())
                })?;

            // Then, a pass to tell llvm about all of the possible fns
            debug!("Registering function types");
            self.functions.iter().try_for_each(|(name, (header, _))| {
                trace!("Registering fn type {}", name);
                let params =
                    header
                        .get_params()
                        .iter()
                        .try_fold(vec![], |params, (_, param)| {
                            Result::Ok(params.fpush(BasicMetadataTypeEnum::from(
                                param.as_llvm_type(&context, &module)?,
                            )))
                        })?;
                let fn_t = match header.get_ret_type() {
                    None => context.void_type().fn_type(&params, false),
                    Some(pqt) => pqt.as_llvm_type(&context, &module)?.fn_type(&params, false),
                };
                module.add_function(name, fn_t, None);
                Result::Ok(())
            })?;

            // Finally, a pass to build all of the code inside the functions
            debug!("Registering function definitions");
            self.functions
                .iter()
                .try_for_each(|(name, (header, nib))| {
                    trace!("Registering definition of function {}", name);
                    nib.clone().commit(&self, &context, &module, &header)
                })?;
            Result::Ok(module)
        });

        match result {
            Err(e) => {
                ("An error occured building llvm IR: {:?}", e);
                // Output the quill to a file for debugging
                let out_path = out_path.with_extension("qir");
                fs::write(&out_path, format!("{:#?}", self))?;
                Err(QuillError::QuillFailed(out_path))
            }
            Ok(module) => {
                let module = match module {
                    Err(e) => {
                        error!("An error occured building llvm IR: {:?}", e);
                        // Output the quill to a file for debugging
                        let out_path = out_path.with_extension("qir");
                        fs::write(&out_path, format!("{:#?}", self))?;
                        Err(QuillError::QuillFailed(out_path))
                    }
                    Ok(module) => Ok(module),
                }?;
                if verify {
                    debug!("Verifying generated IR");
                    // Use unwrap because of weird thread-safety compiler checks
                    module.verify().unwrap();
                }

                // Output the generated artifact to a file
                let out_path = match out_path.extension() {
                    Some(_) => PathBuf::from(out_path),
                    None => out_path.with_extension(artifact_type.get_extension()),
                };
                match artifact_type {
                    ArtifactType::LIR => module.print_to_file(out_path).unwrap(),
                    ArtifactType::Bitcode => {
                        module.write_bitcode_to_path(&out_path);
                    }
                    _ => unreachable!(),
                };
                Ok(())
            }
        }
    }

    ///
    /// Registers a struct type with the quill
    /// Struct types have fields enumerated by index, not by name
    ///
    /// Params:
    /// * `name` - The name of the struct type
    /// * `fields` - The fields that this struct type should contain
    ///
    pub fn register_struct_type(&mut self, name: String, fields: HashMap<String, PolyQuillType>) {
        self.struct_types
            .insert(name.clone(), StructDefinition::new(fields));
    }

    ///
    /// Retrieve a registered struct type by name
    ///
    /// Params:
    /// * `name` - The name of the struct type to get
    ///
    /// Returns:
    /// A copy of the constructed struct type
    ///
    pub(super) fn get_struct_defintion(&self, name: &str) -> Option<&StructDefinition> {
        self.struct_types.get(name)
    }

    ///
    /// Add a function to the quill
    /// The function will have the name and signature declared on the Nib,
    /// and the body of the function will be built by the nib
    ///
    /// Params:
    /// * `nib` - The Nib that contains all of the data necessary for the function
    /// * `params` - The param types of the function
    /// * `returns` - The return type of the function
    ///
    pub fn add_fn(&mut self, nib: FnNib) {
        self.functions
            .insert(nib.get_fn_name().clone(), (nib.get_fn_t().clone(), nib));
    }

    ///
    /// Registers the signature of an external function that must be linked
    ///
    /// Params:
    /// * `name` - The name of the external function
    /// * `args` - The argument types of the external function
    /// * `returns` - The return type of the external function
    ///
    pub fn register_external_fn(&mut self, name: String, t: QuillFnType) -> Result<()> {
        match self.external_functions.insert(name.clone(), t) {
            None => Ok(()),
            Some(_) => Err(QuillError::DuplicateExternalFn(name)),
        }
    }

    pub fn has_fn(&self, name: &str) -> bool {
        match self.functions.get(name) {
            Some(_) => true,
            None => false,
        }
    }
}

pub enum ArtifactType {
    Bitcode,
    LIR,
    QIR,
}

impl ArtifactType {
    fn get_extension(&self) -> &str {
        match self {
            ArtifactType::Bitcode => "bc",
            ArtifactType::LIR => "ll",
            ArtifactType::QIR => "qir",
        }
    }
}

// map of name -> (type, index)
#[derive(Debug, Clone)]
pub(super) struct StructDefinition(HashMap<String, (PolyQuillType, u32)>);
impl StructDefinition {
    ///
    /// Creates a new StructDefinition, with the indexes automatically assigned
    ///
    fn new(types: HashMap<String, PolyQuillType>) -> Self {
        let indexed_types = types.into_iter().enumerate().fold(
            HashMap::new(),
            |indexed_types, (index, (name, q_type))| {
                indexed_types.finsert(name, (q_type, index as u32))
            },
        );
        Self(indexed_types)
    }

    pub(super) fn get_index(&self, name: &str) -> Result<u32> {
        Ok(self
            .0
            .get(name)
            .ok_or(QuillError::StructHasNoField(name.into()))?
            .1)
    }

    fn get_types(&self) -> Vec<PolyQuillType> {
        let mut pqts: Vec<Option<PolyQuillType>> =
            (0..self.0.len()).fold(vec![], |pqts, _| pqts.fpush(None));
        self.0
            .iter()
            .for_each(|(_, (pqt, i))| pqts.replace(*i as usize, Some(pqt.clone())));
        pqts.into_iter().map(|pqt_opt| pqt_opt.unwrap()).collect()
    }
}
