use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabType, FuncSignature, Ident, PosParam, StructId};
use crate::quill::{FnNib, IntCmpType, Nib, PolyQuillType, Quill, QuillBoolType, QuillFloatType, QuillFnType, QuillIntType, QuillListType, QuillPointerType, QuillStructType, QuillVoidType};
use crate::util::{bool_struct_name, capacity_field_name, format_i_c_name, get_fn_name, int_struct_name, length_field_name, list_struct_name, ListFunctional, magic_main_func_name, main_func_name, MapFunctional, new_list_name, operator_add_name, primitive_field_name, printf_c_name, printf_crab_name, resize_name, string_struct_name, to_string_name};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::TryInto;

type FnDefFn = fn(&mut Quill, &mut FnNib, caller_opt: Option<StructId>, tmpls: Vec<StructId>) -> Result<()>;
type FnNameMap = HashMap<Ident, FnDefFn>;
type StrctNameMap = HashMap<Ident, HashMap<Ident, StructTypeResolver>>;

lazy_static! {
    /// A map of the names of each of our function builtins to the function that generates the ir for that builtin
    static ref FN_BUILTIN_NAME_MAP: FnNameMap = init_builtin_fn_map();

    /// A map of the names of each of our struct builtins to the function that generates the ir for that builtin
    static ref STRCT_BUILTIN_NAME_MAP: StrctNameMap = init_builtin_strct_map();
}

///
/// Initialize the builtin function map
/// The builtin name map is populated with *hashed* function names -> function body generator
///
fn init_builtin_fn_map() -> FnNameMap {
    let map: FnNameMap = HashMap::from([
        (mangle_fn_name(&operator_add_name(), &int_struct_name()), add_int as FnDefFn),
        (mangle_fn_name(&to_string_name(), &int_struct_name()), format_i as FnDefFn),
        (mangle_fn_name(&printf_crab_name(), ""), add_printf as FnDefFn),
        (mangle_fn_name(&new_list_name(), ""), add_new_list as FnDefFn),
        (mangle_fn_name(&operator_add_name(), &list_struct_name()), list_add_fn as FnDefFn),
        (mangle_fn_name(&get_fn_name(), &list_struct_name(),), list_get_fn as FnDefFn),
    ]);
    map
}

fn mangle_fn_name(name: &str, caller_name: &str) -> Ident {
    format!("-{}-{}", name, caller_name)
}

///
/// Init the builtin struct definition map
///
fn init_builtin_strct_map() -> StrctNameMap {
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

pub(super) fn add_builtin_definition(peter: &mut Quill, nib: &mut FnNib, caller_opt: Option<StructId>, tmpls: Vec<StructId>) -> Result<()> {
    let fn_name = nib
        .get_fn_name()
        .split("-")
        .skip(2)
        .next()
        .unwrap();
    let caller_name = caller_opt
        .clone()
        .or(Some(StructId::from_name(Ident::from(""))))
        .unwrap()
        .name;
    FN_BUILTIN_NAME_MAP
        .get(&mangle_fn_name(fn_name, &caller_name))
        .ok_or(CompileError::CouldNotFindFunction(
            String::from(fn_name),
        ))?(peter, nib, caller_opt, tmpls)
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

fn add_printf(peter: &mut Quill, nib: &mut FnNib, _: Option<StructId>, _: Vec<StructId>) -> Result<()> {
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

fn add_new_list(_: &mut Quill, nib: &mut FnNib, _: Option<StructId>, tmpls: Vec<StructId>) -> Result<()> {
    let capacity_param = nib.get_fn_param(
        capacity_field_name(),
        QuillPointerType::new(QuillStructType::new(int_struct_name()))
    );
    let capacity = nib.get_value_from_struct(
        &capacity_param,
        primitive_field_name(),
        QuillIntType::new(64),
    )?;
    let t_star = nib.add_malloc(
        QuillListType::new_var_length(
            QuillPointerType::new(
                QuillStructType::new(tmpls[0].mangle())
            ),
            capacity.clone(),
        )
    );
    let list = nib.add_malloc(
        QuillStructType::new(
            StructId { name: list_struct_name(), tmpls: vec![tmpls[0].clone()] }.mangle()
        )
    );
    nib.set_value_in_struct(&list, primitive_field_name(), &t_star)?;
    let zero = nib.const_int(64, 0);
    nib.set_value_in_struct(&list, length_field_name(), &zero)?;
    nib.set_value_in_struct(&list, capacity_field_name(), &capacity)?;
    nib.add_return(Some(&list));
    Ok(())
}

fn list_add_fn(peter: &mut Quill, nib: &mut FnNib, caller: Option<StructId>, _: Vec<StructId>) -> Result<()> {
    let caller = caller.unwrap();
    let list = nib.get_fn_param(
        Ident::from("self"),
        QuillPointerType::new(QuillStructType::new(
        StructId { name: list_struct_name(), tmpls: caller.tmpls.clone() }.mangle()
    )));
    let length = nib.get_value_from_struct(&list, length_field_name(), QuillIntType::new(64))?;
    let capacity = nib.get_value_from_struct(&list, capacity_field_name(), QuillIntType::new(64))?;

    // Resize the array if needed
    let mut then_nib = nib.create_child();
    let new_capacity = then_nib.int_add(&capacity, &capacity)?;
    then_nib.set_value_in_struct(&list, capacity_field_name(), &new_capacity)?;
    let new_t_star = then_nib.add_malloc(
        QuillListType::new_var_length(
            QuillPointerType::new(
                QuillStructType::new(caller.tmpls[0].mangle())
            ),
            new_capacity_value.clone(),
        )
    );
    let old_t_star = then_nib.get_value_from_struct(
        &list,
        primitive_field_name(),
        QuillPointerType::new(QuillStructType::new(caller.tmpls[0].mangle())),
    )?;
    then_nib.list_copy(&old_t_star, &new_t_star, &capacity_value)?;
    then_nib.set_value_in_struct(&list, primitive_field_name(), &new_t_star)?;
    then_nib.free(old_t_star);
    then_nib.add_branch(after_nib);

    // Do the actual if statement now
    let cond = nib.int_cmp(&length, &capacity, IntCmpType::EQ)?;
    nib.add_cond_branch(&cond, then_nib, None);

    // Continue the rest of the function
    let element = nib.get_fn_param(
        Ident::from("element"),
        QuillStructType::new(
            caller.tmpls[0].mangle()
        )
    );
    let t_star = nib.get_value_from_struct(
        &list,
        primitive_field_name(),
        QuillPointerType::new(QuillStructType::new(caller.tmpls[0].mangle())),
    )?;
    nib.set_list_value(&t_star, &element, &length)?;
    let one = nib.const_int(64, 1);
    let new_len = nib.int_add(&length, &one)?;
    nib.set_value_in_struct(&list, length_field_name(), &new_len)?;

    // Return nothing
    nib.add_return(QuillFnType::void_return_value());

    
    Ok(())
}

fn list_get_fn(_: &mut Quill, nib: &mut FnNib, caller: Option<StructId>, _: Vec<StructId>) -> Result<()> {
    let caller = caller.unwrap();
    let list = nib.get_fn_param(
        Ident::from("self"),
        QuillPointerType::new(QuillStructType::new(
            StructId { name: list_struct_name(), tmpls: caller.tmpls.clone() }.mangle()
        )));
    let index = nib.get_fn_param(
        Ident::from("index"),
        QuillPointerType::new(
            QuillStructType::new(
                int_struct_name()
            )
        )
    );
    let t_star = nib.get_value_from_struct(
        &list,
        primitive_field_name(),
        QuillPointerType::new(QuillStructType::new(caller.tmpls[0].mangle())),
    )?;
    let index_value = nib.get_value_from_struct(&index, primitive_field_name(), QuillIntType::new(64))?;
    let value = nib.get_list_value(&t_star, &index_value, t_star.get_type().get_inner_type())?;
    nib.add_return(Some(&value));
    Ok(())
}

fn list_resize_fn(_: &mut Quill, nib: &mut FnNib, caller: Option<StructId>, _: Vec<StructId>) -> Result<()> {
    let caller = caller.unwrap();
    let list = nib.get_fn_param(
        Ident::from("self"),
        QuillPointerType::new(QuillStructType::new(
            StructId { name: list_struct_name(), tmpls: caller.tmpls.clone() }.mangle()
        )));
    let capacity = nib.get_value_from_struct(
        &list,
        capacity_field_name(),
        QuillPointerType::new(
            QuillStructType::new(
                int_struct_name()
            )
        ))?;
    let capacity_value = nib.get_value_from_struct(&capacity, primitive_field_name(), QuillIntType::new(64))?;
    let new_capacity_value = nib.int_add(&capacity_value, &capacity_value)?;
    nib.set_value_in_struct(&capacity, primitive_field_name(), &new_capacity_value)?;
    let new_t_star = nib.add_malloc(
        QuillListType::new_var_length(
            QuillPointerType::new(
                QuillStructType::new(caller.tmpls[0].mangle())
            ),
            new_capacity_value.clone(),
        )
    );
    let old_t_star = nib.get_value_from_struct(
        &list,
        primitive_field_name(),
        QuillPointerType::new(QuillStructType::new(caller.tmpls[0].mangle())),
    )?;
    nib.list_copy(&old_t_star, &new_t_star, &capacity_value)?;
    nib.set_value_in_struct(&list, primitive_field_name(), &new_t_star)?;

    nib.free(old_t_star);

    nib.add_return(QuillFnType::void_return_value());
    Ok(())
}

fn add_int(_: &mut Quill, nib: &mut FnNib, _: Option<StructId>, _: Vec<StructId>) -> Result<()> {
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

    let result_int = nib.int_add(&self_int, &other_int)?;

    let ret_val = nib.add_malloc(QuillStructType::new(int_name_mangled()));
    nib.set_value_in_struct(&ret_val, primitive_field_name(), &result_int)?;
    nib.add_return(Some(&ret_val));

    Ok(())
}

fn format_i(peter: &mut Quill, nib: &mut FnNib, _: Option<StructId>, _: Vec<StructId>) -> Result<()> {
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
    nib.set_value_in_struct(&ret_val, primitive_field_name(), &char_star)?;

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
        tmpls: vec![],
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
