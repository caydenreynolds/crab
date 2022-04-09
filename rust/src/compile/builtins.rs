use crate::compile::CompileError::MallocErr;
use crate::compile::{Codegen, CompileError, CrabValueType, FnManager, Functiongen, Result};
use crate::parse::ast::{CrabType, FnParam, FuncSignature, Ident};
use crate::util::{
    format_i_c_name, int_struct_name, main_func_name, mangle_function_name, new_string_name,
    operator_add_name, printf_c_name, printf_crab_name, string_type_name, to_string_name,
};
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue, IntValue, PointerValue};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// A map of the names of each of our compiler builtins to the function that generates the ir for that builtin
    static ref BUILTIN_NAME_MAP: HashMap<String, fn(&mut Functiongen)->Result<()>> = init_builtin_name_map();
}

///
/// Initialize the builtin name map
/// The builtin name map is populated with *hashed* function names -> function body generator
///
fn init_builtin_name_map() -> HashMap<String, fn(&mut Functiongen) -> Result<()>> {
    let mut map: HashMap<String, fn(&mut Functiongen) -> Result<()>> = HashMap::new();

    let int_add = mangle_function_name(&operator_add_name(), Some(&int_struct_name()));
    map.insert(int_add, add_int);
    let int_to_str = mangle_function_name(&to_string_name(), Some(&int_struct_name()));
    map.insert(int_to_str, format_i);

    map
}

pub fn add_builtin_definition(funcgen: &mut Functiongen) -> Result<()> {
    BUILTIN_NAME_MAP
        .get(&funcgen.name)
        .ok_or(CompileError::CouldNotFindFunction(funcgen.name.clone()))?(funcgen)
}

pub fn add_builtins(codegen: &mut Codegen, fns: &mut FnManager) -> Result<()> {
    printf_c(codegen, fns)?;
    printf_crab(codegen, fns)?;
    new_str(codegen, fns)?;

    Ok(())
}

pub fn add_main_func(codegen: &mut Codegen, fns: &mut FnManager) -> Result<()> {
    let fn_name = main_func_name();
    let signature = FuncSignature {
        name: fn_name.clone(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![],
        named_params: vec![],
    };
    fns.register_builtin(signature.clone(), false, None)?;

    let main_fn_value = codegen
        .get_module()
        .get_function(&mangle_function_name(&main_func_name(), None))
        .unwrap();

    let (_, builder) = begin_func(codegen, &signature)?;
    let csv = builder.build_call(main_fn_value, &[], "call");
    let crab_value = CrabValueType::from_call_site_value(csv, CrabType::STRUCT(int_struct_name()));
    let loaded = builder.build_load(crab_value.try_as_struct_value()?, "loaded");

    let result = builder
        .build_extract_value(loaded.into_struct_value(), 0, "result")
        .unwrap();
    builder.build_return(Some(&result));

    Ok(())
}

fn new_str(codegen: &mut Codegen, fns: &mut FnManager) -> Result<()> {
    let signature = FuncSignature {
        name: new_string_name(),
        return_type: CrabType::STRUCT(string_type_name()),
        unnamed_params: vec![
            FnParam {
                name: Ident::from("who_cares"),
                crab_type: CrabType::STRING,
            },
            FnParam {
                name: Ident::from("i_sure_dont"),
                crab_type: CrabType::UINT64,
            },
        ],
        named_params: vec![],
    }
    .with_mangled_name();
    fns.register_builtin(signature.clone(), true, Some(Linkage::External))?;

    let (fn_value, builder) = begin_func(codegen, &signature)?;

    // Get the values of the function parameters
    let src = fn_value.get_nth_param(0).unwrap().into_pointer_value();
    let len = fn_value.get_nth_param(1).unwrap().into_int_value();

    //TODO: free
    let strct = builder
        .build_malloc(
            codegen
                .get_module()
                .get_struct_type(&string_type_name())
                .unwrap(),
            "strct",
        )
        .or(Err(CompileError::MallocErr(String::from(
            "builtins::new_str",
        ))))?;
    //TODO: free
    let buf = builder
        .build_array_malloc(codegen.get_context().i8_type(), len, "buf")
        .or(Err(CompileError::MallocErr(String::from(
            "builtins::new_str",
        ))))?;
    let dest_ptr = builder
        .build_struct_gep(strct, 0, "buf_ptr")
        .or(Err(CompileError::Gep(String::from("builtins::new_str"))))?;

    builder.build_memcpy(buf, 1, src, 1, len).unwrap();
    builder.build_store(dest_ptr, buf);
    builder.build_return(Some(&strct));

    Ok(())
}

fn printf_c(_codegen: &mut Codegen, fns: &mut FnManager) -> Result<()> {
    let signature = FuncSignature {
        name: printf_c_name(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![FnParam {
            name: Ident::from("str"),
            crab_type: CrabType::STRING,
        }],
        named_params: vec![],
    };
    fns.register_builtin(signature.clone(), true, Some(Linkage::External))?;
    Ok(())
}

fn printf_crab(codegen: &mut Codegen, fns: &mut FnManager) -> Result<()> {
    let signature = FuncSignature {
        name: printf_crab_name(),
        return_type: CrabType::VOID,
        unnamed_params: vec![FnParam {
            name: Ident::from("str"),
            crab_type: CrabType::STRUCT(string_type_name()),
        }],
        named_params: vec![],
    }
    .with_mangled_name();
    fns.register_builtin(signature.clone(), false, None)?;

    let printf_c_fn_value = codegen.get_module().get_function(&printf_c_name()).unwrap();

    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let arg = fn_value.get_nth_param(0).unwrap().into_pointer_value();
    let buf = builder
        .build_struct_gep(arg, 0, "gep")
        .or(Err(CompileError::Gep(String::from(
            "builtins::printf_crab",
        ))))?;
    let buf_ptr = builder.build_load(buf, "buf_ptr");
    let args = [BasicMetadataValueEnum::from(buf_ptr)];
    builder.build_call(printf_c_fn_value, &args, "call");
    builder.build_return(None);
    Ok(())
}

fn format_i(funcgen: &mut Functiongen) -> Result<()> {
    // The signature of the underlying c functino (in crabbuiltins.lib) that we want to link to
    let format_i_c_signature = FuncSignature {
        name: format_i_c_name(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![
            FnParam {
                name: Ident::from("dest"),
                crab_type: CrabType::STRING,
            },
            FnParam {
                name: Ident::from("data"),
                crab_type: CrabType::UINT64,
            },
        ],
        named_params: vec![],
    };
    funcgen
        .fns
        .register_builtin(format_i_c_signature, false, None)?;

    let format_i_c_value = funcgen.module.get_function(&format_i_c_name()).unwrap();

    //TODO: free
    let strct = funcgen
        .builder
        .build_malloc(
            funcgen.module.get_struct_type(&string_type_name()).unwrap(),
            "strct",
        )
        .or(Err(CompileError::MallocErr(String::from(
            "builtins::new_str",
        ))))?;
    //An arbitrary len was chosen here, but we should never be able to exceed 25 characters
    let len = funcgen.context.i32_type().const_int(25, false);
    //TODO: free
    let buf = funcgen
        .builder
        .build_array_malloc(funcgen.context.i8_type(), len, "buf")
        .or(Err(CompileError::MallocErr(String::from(
            "builtins::new_str",
        ))))?;
    let dest_ptr = funcgen
        .builder
        .build_struct_gep(strct, 0, "buf_ptr")
        .or(Err(CompileError::Gep(String::from("builtins::new_str"))))?;
    funcgen.builder.build_store(dest_ptr, buf);

    let int_arg = funcgen
        .fn_value
        .get_nth_param(0)
        .unwrap()
        .into_pointer_value();
    let int_val = extract_int(funcgen, int_arg)?;
    let args = [
        BasicMetadataValueEnum::from(buf),
        BasicMetadataValueEnum::from(int_val),
    ];

    funcgen.builder.build_call(format_i_c_value, &args, "call");
    funcgen.builder.build_return(Some(&strct));

    Ok(())
}

fn add_int(funcgen: &mut Functiongen) -> Result<()> {
    let arg1 = funcgen
        .fn_value
        .get_nth_param(0)
        .unwrap()
        .into_pointer_value();
    let arg2 = funcgen
        .fn_value
        .get_nth_param(1)
        .unwrap()
        .into_pointer_value();

    let lhs = extract_int(funcgen, arg1)?;
    let rhs = extract_int(funcgen, arg2)?;

    let result_val = funcgen.builder.build_int_add(lhs, rhs, "result_val");
    let result = funcgen
        .builder
        .build_malloc(
            funcgen
                .module
                .get_struct_type(&int_struct_name())
                .ok_or(CompileError::StructDoesNotExist(int_struct_name()))?,
            "result",
        )
        .or(Err(MallocErr(String::from("builtins::add_int"))))?;
    let result_ptr = funcgen
        .builder
        .build_struct_gep(result, 0, "result_ptr")
        .or(Err(CompileError::Gep(String::from("builtins::add_int"))))?;

    funcgen.builder.build_store(result_ptr, result_val);
    funcgen.builder.build_return(Some(&result));

    Ok(())
}

///
/// Extracts the underlying uint64 value from a pointer to an int struct
///
fn extract_int<'a, 'b, 'ctx>(
    funcgen: &mut Functiongen<'a, 'b, 'ctx>,
    value: PointerValue<'ctx>,
) -> Result<IntValue<'ctx>> {
    let ptr = funcgen
        .builder
        .build_struct_gep(value, 0, "ptr")
        .or(Err(CompileError::Gep(String::from("builtins::add_int"))))?;
    let value = funcgen.builder.build_load(ptr, "value").into_int_value();
    Ok(value)
}

fn begin_func<'ctx>(
    codegen: &'ctx Codegen,
    sig: &FuncSignature,
) -> Result<(FunctionValue<'ctx>, Builder<'ctx>)> {
    let fn_value = codegen.get_module().get_function(&sig.name).unwrap();
    let basic_block = codegen.get_context().append_basic_block(fn_value, "entry");
    let builder = codegen.get_context().create_builder();
    builder.position_at_end(basic_block);
    Ok((fn_value, builder))
}
