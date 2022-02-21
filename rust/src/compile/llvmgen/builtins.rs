use crate::compile::llvmgen::{Codegen, CrabValueType};
use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabType, FnParam, FuncSignature, Ident};
use crate::parse::{
    int_struct_name, main_func_name, mangle_function_name, new_string_name, string_type_name,
};
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue};
use inkwell::AddressSpace;

fn printf_c_name() -> Ident {
    Ident::from("printf")
}
fn printf_crab_name() -> Ident {
    Ident::from("__printf__")
}
fn format_i_c_name() -> Ident {
    Ident::from("__c_format_i__")
}
fn format_i_name() -> Ident {
    Ident::from("__format_i__")
}
fn add_int_name() -> Ident {
    Ident::from("__add_int__")
}

pub fn add_builtins(codegen: &mut Codegen) -> Result<()> {
    printf_c(codegen)?;
    printf_crab(codegen)?;
    format_i_c(codegen)?;
    format_i(codegen)?;
    add_int(codegen)?;
    new_str(codegen)?;

    Ok(())
}

pub fn add_main_func(codegen: &mut Codegen) -> Result<()> {
    let fn_name = main_func_name();
    let signature = FuncSignature {
        name: fn_name.clone(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![],
        named_params: vec![],
    };

    let main_fn_value = codegen
        .get_module()
        .get_function(&mangle_function_name(&main_func_name(), None))
        .unwrap();

    codegen.register_function(signature.clone(), true, None)?;

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

fn new_str(codegen: &mut Codegen) -> Result<()> {
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

    codegen.register_function(signature.clone(), true, Some(Linkage::External))?;
    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let strct = builder.build_alloca(
        codegen
            .get_module()
            .get_struct_type(&string_type_name())
            .unwrap(),
        "strct",
    );
    let buf = builder
        .build_struct_gep(strct, 0, "buf")
        .or(Err(CompileError::Gep(String::from("builtins::new_str"))))?;
    let src = fn_value.get_nth_param(0).unwrap().into_pointer_value();
    let len = fn_value.get_nth_param(1).unwrap().into_int_value();
    builder.build_memcpy(buf, 1, src, 1, len).unwrap();
    builder.build_return(Some(&strct));

    Ok(())
}

fn printf_c(codegen: &mut Codegen) -> Result<()> {
    let signature = FuncSignature {
        name: printf_c_name(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![FnParam {
            name: Ident::from("str"),
            crab_type: CrabType::STRING,
        }],
        named_params: vec![],
    };
    codegen.register_function(signature, true, Some(Linkage::External))?;
    Ok(())
}

fn printf_crab(codegen: &mut Codegen) -> Result<()> {
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

    let printf_c_fn_value = codegen.get_module().get_function(&printf_c_name()).unwrap();

    codegen.register_function(signature.clone(), true, None)?;

    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let fmt_str = builder.build_global_string_ptr("%s", "fmtstr");
    let arg = fn_value.get_nth_param(0).unwrap().into_pointer_value();
    let buf = builder
        .build_struct_gep(arg, 0, "gep")
        .or(Err(CompileError::Gep(String::from(
            "builtins::printf_crab",
        ))))?;
    let args = [
        BasicMetadataValueEnum::from(fmt_str.as_pointer_value()),
        BasicMetadataValueEnum::from(buf),
    ];
    builder.build_call(printf_c_fn_value, &args, "call");
    builder.build_return(None);
    Ok(())
}

fn format_i_c(codegen: &mut Codegen) -> Result<()> {
    let signature = FuncSignature {
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
    codegen.register_function(signature, true, Some(Linkage::External))?;
    Ok(())
}

fn format_i(codegen: &mut Codegen) -> Result<()> {
    let signature = FuncSignature {
        name: format_i_name(),
        return_type: CrabType::STRUCT(string_type_name()),
        unnamed_params: vec![FnParam {
            name: Ident::from("name"),
            crab_type: CrabType::UINT64,
        }],
        named_params: vec![],
    }
    .with_mangled_name();

    let format_i_c_value = codegen
        .get_module()
        .get_function(&format_i_c_name())
        .unwrap();
    codegen.register_function(signature.clone(), true, None)?;
    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let str = builder.build_alloca(
        codegen
            .get_module()
            .get_struct_type(&string_type_name())
            .unwrap(),
        "str",
    );
    let buf = builder
        .build_struct_gep(str, 0, "buf")
        .or(Err(CompileError::Gep(String::from("builtins::format_i"))))?;
    let buf_ptr = builder.build_bitcast(
        buf,
        codegen
            .get_context()
            .i8_type()
            .ptr_type(AddressSpace::Generic),
        "buf_ptr",
    );
    let int = fn_value.get_nth_param(0).unwrap().into_int_value();
    let args = [
        BasicMetadataValueEnum::from(buf_ptr),
        BasicMetadataValueEnum::from(int),
    ];

    builder.build_call(format_i_c_value, &args, "call");
    builder.build_return(Some(&str));

    Ok(())
}

fn add_int(codegen: &mut Codegen) -> Result<()> {
    let fn_name = add_int_name();
    let signature = FuncSignature {
        name: fn_name.clone(),
        return_type: CrabType::UINT64,
        unnamed_params: vec![
            FnParam {
                name: Ident::from("lhs"),
                crab_type: CrabType::UINT64,
            },
            FnParam {
                name: Ident::from("rhs"),
                crab_type: CrabType::UINT64,
            },
        ],
        named_params: vec![],
    }
    .with_mangled_name();

    codegen.register_function(signature.clone(), true, None)?;

    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let result = builder.build_int_add(
        fn_value.get_nth_param(0).unwrap().into_int_value(),
        fn_value.get_nth_param(1).unwrap().into_int_value(),
        "addition",
    );
    builder.build_return(Some(&result));

    Ok(())
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
