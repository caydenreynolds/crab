use crate::compile::llvmgen::{Codegen, CrabValueType};
use crate::compile::{CompileError, Result};
use crate::parse::ast::{CrabType, FnParam, FuncSignature, Ident};
use crate::parse::{
    int_struct_name, main_func_name, mangle_function_name, new_string_name, string_type_name,
};
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::values::{BasicMetadataValueEnum, FunctionValue};
use inkwell::IntPredicate;

fn printf_c_name() -> Ident {
    Ident::from("printf")
}
fn printf_crab_name() -> Ident {
    Ident::from("__printf__")
}
fn add_int_name() -> Ident {
    Ident::from("__add_int__")
}

pub fn add_builtins(codegen: &mut Codegen) -> Result<()> {
    printf_c(codegen)?;
    printf_crab(codegen)?;
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

    // let src = fn_value.get_nth_param(0).unwrap().into_pointer_value();

    // // Loop until we find the null terminator
    // let i_0 = codegen.get_context().i32_type().const_int(4294967295, false); // Use maximum value for a 32 bit integer so it immediately overflows to zero
    // let i = builder.build_alloca(codegen.get_context().i32_type(), "i");
    // builder.build_store(i, i_0);
    // let loop_block = codegen.get_context().append_basic_block(fn_value, "loop");
    // let after_loop_block = codegen.get_context().append_basic_block(fn_value, "after_loop");
    // builder.build_unconditional_branch(loop_block);
    // builder.position_at_end(loop_block);
    // let loaded_i = builder.build_load(i, "loaded_i").into_int_value();
    // let added_i = builder.build_int_add(loaded_i, codegen.get_context().i32_type().const_int(1, false), "added_i");
    // builder.build_store(i, added_i);
    // unsafe {
    //     let char_ptr = builder.build_gep(src, &[added_i], "char_ptr");
    //     let char = builder.build_load(char_ptr, "char").into_int_value();
    //     let char_0 = codegen.get_context().i8_type().const_int(0, false);
    //     let char_cmp = builder.build_int_compare(IntPredicate::EQ, char, char_0, "char_cmp");
    //     builder.build_conditional_branch(char_cmp,after_loop_block, loop_block);
    // }
    //
    // // Perform the copy
    // builder.position_at_end(after_loop_block);
    // let buf_type = BasicTypeEnum::ArrayType(codegen.get_context().i8_type().array_type(string_buf_len()));
    // let dest = builder.build_alloca(buf_type, "name");
    // let i_val = builder.build_load(i, "i_val").into_int_value();
    // builder.build_memcpy(dest, 1, src, 1, i_val).unwrap();
    // let loaded = builder.build_load(dest, "loaded");
    // builder.build_return(Some(&loaded));

    Ok(())
}

fn printf_c(codegen: &mut Codegen) -> Result<()> {
    let signature = FuncSignature {
        name: printf_c_name(),
        return_type: CrabType::FLOAT,
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
