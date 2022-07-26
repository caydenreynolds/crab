use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabType, FuncSignature, Ident, PosParam, StructId};
use crate::quill::{
    FnNib, Nib, PolyQuillType, Quill, QuillFloatType, QuillFnType, QuillIntType, QuillListType,
    QuillPointerType, QuillStructType, QuillVoidType,
};
use crate::util::{
    bool_struct_name, format_i_c_name, int_struct_name, main_func_name,
    operator_add_name, primitive_field_name, printf_c_name, printf_crab_name,
    string_type_name, to_string_name,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::default;
use serde::de::Unexpected::Str;

lazy_static! {
    /// A map of the names of each of our function builtins to the function that generates the ir for that builtin
    static ref FN_BUILTIN_NAME_MAP: HashMap<Ident, fn(&mut Quill, &mut FnNib)->Result<()>> = init_builtin_fn_map();

    /// A map of the names of each of our struct builtins to the function that generates the ir for that builtin
    static ref STRCT_BUILTIN_NAME_MAP: HashMap<Ident, HashMap<Ident, PolyQuillType>> = init_builtin_strct_map();
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
        named_params: default::Default(),
        caller_id: Some(StructId::from_name(int_struct_name()))
    }.mangled();
    map.insert(int_add.name, add_int);

    let int_to_str = FuncSignature {
        name: to_string_name(),
        return_type: CrabType::SIMPLE(string_type_name()),
        pos_params: vec![
            PosParam {
                name: Ident::from("self"),
                crab_type: CrabType::SIMPLE(int_struct_name()),
            },
        ],
        named_params: default::Default(),
        caller_id: Some(StructId::from_name(int_struct_name())),
    }.mangled();
    map.insert(int_to_str.name, format_i);

    let printf = FuncSignature {
        name: printf_crab_name(),
        return_type: CrabType::VOID,
        pos_params: vec![
            PosParam {
                name: Ident::from("str"),
                crab_type: CrabType::SIMPLE(string_struct_name()),
            },
        ],
        named_params: default::Default(),
        caller_id: None,
    }.mangled();
    map.insert(printf.name, add_printf);

    map
}

///
/// Init the builtin struct definition map
///
fn init_builtin_strct_map() -> HashMap<Ident, HashMap<String, PolyQuillType>> {
    HashMap::from([
        (
            int_struct_name(),
            HashMap::from([(primitive_field_name(), QuillIntType::new(64).into())]),
        ),
        (
            bool_struct_name(),
            HashMap::from([(primitive_field_name(), QuillIntType::new(1).into())]),
        ),
        (
            string_type_name(),
            HashMap::from([(
                primitive_field_name(),
                QuillPointerType::new(QuillIntType::new(8)).into(),
            )]),
        ),
    ])
}

pub(super) fn add_builtin_definition(peter: &mut Quill, nib: &mut FnNib) -> Result<()> {
    FN_BUILTIN_NAME_MAP
        .get(nib.get_fn_name())
        .ok_or(CompileError::CouldNotFindFunction(
            nib.get_fn_name().clone(),
        ))?(peter, nib)
}

pub(super) fn get_builtin_strct_definition(name: &str) -> Result<&HashMap<String, PolyQuillType>> {
    STRCT_BUILTIN_NAME_MAP
        .get(name)
        .ok_or(CompileError::NotAStruct(
            StructId::from_name(Ident::from(name)),
            String::from("builtins::get_builtin_strct_definition"),
        ))
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
        QuillPointerType::new(QuillStructType::new(string_type_name())),
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
        QuillPointerType::new(QuillStructType::new(int_struct_name())),
    );
    let other_arg = nib.get_fn_param(
        String::from("other"),
        QuillPointerType::new(QuillStructType::new(int_struct_name())),
    );

    let self_int =
        nib.get_value_from_struct(&self_arg, primitive_field_name(), QuillIntType::new(64))?;
    let other_int =
        nib.get_value_from_struct(&other_arg, primitive_field_name(), QuillIntType::new(64))?;

    let result_int = nib.int_add(self_int, other_int)?;

    let ret_val = nib.add_malloc(QuillStructType::new(int_struct_name()));
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
        QuillPointerType::new(QuillStructType::new(int_struct_name())),
    );
    let self_int =
        nib.get_value_from_struct(&self_arg, primitive_field_name(), QuillIntType::new(64))?;

    let char_star = nib.add_malloc(QuillListType::new_const_length(QuillIntType::new(8), 50));
    let ret_val = nib.add_malloc(QuillStructType::new(string_type_name()));
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

    let result = nib.add_fn_call(
        mangle_function_name(&main_func_name(), None),
        vec![],
        QuillPointerType::new(QuillStructType::new(int_struct_name())),
    );
    let result_value =
        nib.get_value_from_struct(&result, primitive_field_name(), QuillIntType::new(64))?;
    nib.add_return(Some(&result_value));

    peter.add_fn(nib);

    Ok(())
}
