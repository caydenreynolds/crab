use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabType, FuncSignature, Ident, PosParam, StructId};
use crate::quill::{FnNib, Nib, PolyQuillType, Quill, QuillBoolType, QuillFloatType, QuillFnType, QuillIntType, QuillListType, QuillPointerType, QuillStructType, QuillVoidType};
use crate::util::{bool_struct_name, capacity_field_name, format_i_c_name, int_struct_name, length_field_name, list_struct_name, ListFunctional, magic_main_func_name, main_func_name, MapFunctional, operator_add_name, primitive_field_name, printf_c_name, printf_crab_name, string_struct_name, to_string_name};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::TryInto;

lazy_static! {
    /// A map of the names of each of our function builtins to the function that generates the ir for that builtin
    static ref FN_BUILTIN_NAME_MAP: HashMap<Ident, fn(&mut Quill, &mut FnNib)->Result<()>> = init_builtin_fn_map();

    /// A map of the names of each of our struct builtins to the function that generates the ir for that builtin
    static ref STRCT_BUILTIN_NAME_MAP: HashMap<Ident, HashMap<Ident, StructTypeResolver>> = init_builtin_strct_map();
}

///
/// Initialize the builtin function map
/// The builtin name map is populated with *hashed* function names -> function body generator
///
fn init_builtin_fn_map() -> HashMap<Ident, fn(&mut Quill, &mut FnNib) -> Result<()>> {
    let mut map: HashMap<Ident, fn(&mut Quill, &mut FnNib) -> Result<()>> = HashMap::new();

    let int_add = FuncSignature {
        name: operator_add_name(),
        return_type: CrabType::SIMPLE(int_struct_name()),
        pos_params: vec![
            PosParam {
                name: Ident::from("self"),
                crab_type: CrabType::SIMPLE(int_struct_name()),
            },
            PosParam {
                name: Ident::from("other"),
                crab_type: CrabType::SIMPLE(int_struct_name()),
            },
        ],
        named_params: Default::default(),
        caller_id: Some(StructId::from_name(int_struct_name())),
    }
    .mangled();
    map.insert(int_add.name, add_int);

    let int_to_str = FuncSignature {
        name: to_string_name(),
        return_type: CrabType::SIMPLE(string_struct_name()),
        pos_params: vec![PosParam {
            name: Ident::from("self"),
            crab_type: CrabType::SIMPLE(int_struct_name()),
        }],
        named_params: Default::default(),
        caller_id: Some(StructId::from_name(int_struct_name())),
    }
    .mangled();
    map.insert(int_to_str.name, format_i);

    let printf = FuncSignature {
        name: printf_crab_name(),
        return_type: CrabType::VOID,
        pos_params: vec![PosParam {
            name: Ident::from("str"),
            crab_type: CrabType::SIMPLE(string_struct_name()),
        }],
        named_params: Default::default(),
        caller_id: None,
    }
    .mangled();
    map.insert(printf.name, add_printf);

    map
}

///
/// Init the builtin struct definition map
///
fn init_builtin_strct_map() -> HashMap<Ident, HashMap<String, StructTypeResolver>> {
    HashMap::from([
        (
            int_struct_name(),
            HashMap::from([
                (primitive_field_name(), StructTypeResolver::QuillType(QuillIntType::new(64).into())),
            ]),
        ),
        (
            bool_struct_name(),
            HashMap::from([
                (primitive_field_name(), StructTypeResolver::QuillType(QuillIntType::new(1).into())),
            ]),
        ),
        (
            string_struct_name(),
            HashMap::from([(
                primitive_field_name(),
                StructTypeResolver::QuillType(QuillPointerType::new(QuillIntType::new(8)).into()),
            )]),
        ),
        (
            list_struct_name(),
            HashMap::from([
                (primitive_field_name(), StructTypeResolver::TmplTypePtr(0)),
                (length_field_name(), StructTypeResolver::QuillType(QuillIntType::new(64).into())),
                (capacity_field_name(), StructTypeResolver::QuillType(QuillIntType::new(64).into())),
            ]),
        ),
    ])
}

enum StructTypeResolver {
    QuillType(PolyQuillType),
    //TmplType(usize),
    TmplTypePtr(usize),
}

fn resolve_struct(ct: &CrabType, fields: &HashMap<String, StructTypeResolver>) -> Result<HashMap<String, PolyQuillType>> {
    fields
        .iter()
        .try_fold(HashMap::new(), |types, (name, str)| {
            let qt = match str {
                StructTypeResolver::QuillType(qt) => qt.clone(),
                //StructTypeResolver::TmplType(t) => resolve_type(ct, *t)?,
                StructTypeResolver::TmplTypePtr(t) => QuillPointerType::new(resolve_type(ct, *t)?).into(),
            };
            Ok(types.finsert(name.clone(), qt))
        })
}

fn resolve_type(ct: &CrabType, index: usize) -> Result<PolyQuillType> {
    match ct {
        CrabType::TMPL(_, tmpls) => {
            match &tmpls[index] {
                CrabType::VOID => Err(CompileError::VoidType),
                CrabType::PRIM_INT => Ok(QuillIntType::new(64).into()),
                CrabType::PRIM_STR => unimplemented!(),
                CrabType::PRIM_BOOL => Ok(QuillBoolType::new().into()),
                CrabType::SIMPLE(name) => Ok(
                    QuillPointerType::new(
                        QuillStructType::new(
                            StructId::from_name(name.clone()).mangle()
                    )).into()
                ),
                CrabType::TMPL(name, tmpls) => {
                    Ok(
                        QuillPointerType::new(
                            QuillStructType::new(
                            StructId {
                                name: name.clone(),
                                tmpls: tmpls.clone().into_iter().try_fold(vec![], |tmpls, tmpl| {
                                    Result::Ok(tmpls.fpush(tmpl.try_into()?))
                                })?,
                            }.mangle()
                    )).into())
                }
            }
        }
        _ => Err(CompileError::NotATmpl(ct.clone())),
    }
}

pub(super) fn add_builtin_definition(peter: &mut Quill, nib: &mut FnNib) -> Result<()> {
    FN_BUILTIN_NAME_MAP
        .get(nib.get_fn_name())
        .ok_or(CompileError::CouldNotFindFunction(
            nib.get_fn_name().clone(),
        ))?(peter, nib)
}

pub(super) fn get_builtin_strct_definition(ct: &CrabType) -> Result<HashMap<String, PolyQuillType>> {
    let name = ct.try_get_struct_name()?;
    resolve_struct(ct, STRCT_BUILTIN_NAME_MAP
        .get(&name)
        .ok_or(CompileError::NotAStruct(
            StructId::from_name(Ident::from(&name)),
            String::from("builtins::get_builtin_strct_definition"),
        ))?
    )
}

fn add_printf(peter: &mut Quill, nib: &mut FnNib) -> Result<()> {
    // Tell the quill we need to link to the C printf function
    let params = vec![(
        String::from("0"),
        QuillPointerType::new(QuillIntType::new(8)).into(),
    )];
    peter.register_external_fn(
        printf_c_name(),
        QuillFnType::new(Some(QuillFloatType::new()), params),
    )?;

    // Call the C printf function
    let fn_param = nib.get_fn_param(
        String::from("str"),
        QuillPointerType::new(QuillStructType::new(string_struct_name())),
    );
    let char_star = nib.get_value_from_struct(
        &fn_param,
        primitive_field_name(),
        QuillPointerType::new(QuillIntType::new(8)),
    )?;
    nib.add_fn_call(
        printf_c_name(),
        vec![char_star.into()],
        QuillFloatType::new(),
    );

    // Return nothing
    nib.add_return(QuillFnType::void_return_value());

    Ok(())
}

fn add_int(_: &mut Quill, nib: &mut FnNib) -> Result<()> {
    let self_arg = nib.get_fn_param(
        String::from("self"),
        QuillPointerType::new(QuillStructType::new(int_name_mangled())),
    );
    let other_arg = nib.get_fn_param(
        String::from("other"),
        QuillPointerType::new(QuillStructType::new(int_name_mangled())),
    );

    let self_int =
        nib.get_value_from_struct(&self_arg, primitive_field_name(), QuillIntType::new(64))?;
    let other_int =
        nib.get_value_from_struct(&other_arg, primitive_field_name(), QuillIntType::new(64))?;

    let result_int = nib.int_add(self_int, other_int)?;

    let ret_val = nib.add_malloc(QuillStructType::new(int_name_mangled()));
    nib.set_value_in_struct(&ret_val, primitive_field_name(), result_int)?;
    nib.add_return(Some(&ret_val));

    Ok(())
}

fn format_i(peter: &mut Quill, nib: &mut FnNib) -> Result<()> {
    let params = vec![
        (
            String::from("0"),
            QuillPointerType::new(QuillIntType::new(8)).into(),
        ),
        (String::from("1"), QuillIntType::new(64).into()),
    ];
    peter.register_external_fn(
        format_i_c_name(),
        QuillFnType::new(QuillFnType::void_return(), params),
    )?;

    let self_arg = nib.get_fn_param(
        String::from("self"),
        QuillPointerType::new(QuillStructType::new(int_name_mangled())),
    );
    let self_int =
        nib.get_value_from_struct(&self_arg, primitive_field_name(), QuillIntType::new(64))?;

    let char_star = nib.add_malloc(QuillListType::new_const_length(QuillIntType::new(8), 50));
    let ret_val = nib.add_malloc(QuillStructType::new(string_name_mangled()));
    nib.add_fn_call(
        format_i_c_name(),
        vec![char_star.clone().into(), self_int.into()],
        QuillVoidType::new(),
    );
    nib.set_value_in_struct(&ret_val, primitive_field_name(), char_star)?;

    nib.add_return(Some(&ret_val));

    Ok(())
}

pub(super) fn add_main_func(peter: &mut Quill) -> Result<()> {
    let mut nib = FnNib::new(
        main_func_name(),
        QuillFnType::new(Some(QuillIntType::new(64)), vec![]),
    );

    let magic_main_func = FuncSignature {
        name: magic_main_func_name(),
        return_type: CrabType::SIMPLE(int_struct_name()),
        pos_params: Default::default(),
        named_params: Default::default(),
        caller_id: None,
    }
    .mangled();
    let result = nib.add_fn_call(
        magic_main_func.name,
        vec![],
        QuillPointerType::new(QuillStructType::new(int_struct_name())),
    );
    let result_value =
        nib.get_value_from_struct(&result, primitive_field_name(), QuillIntType::new(64))?;
    nib.add_return(Some(&result_value));

    peter.add_fn(nib);

    Ok(())
}

fn int_name_mangled() -> String {
    StructId::from_name(int_struct_name()).mangle()
}

fn string_name_mangled() -> String {
    StructId::from_name(string_struct_name()).mangle()
}
