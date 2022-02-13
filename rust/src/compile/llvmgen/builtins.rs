use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;
use crate::compile::llvmgen::{Codegen, CrabValueType};
use crate::parse::ast::{CrabType, FnParam, FuncSignature, Ident, int_struct_name, internal_main_func_name};
use crate::compile::Result;

pub fn add_builtins(codegen: &mut Codegen) -> Result<()> {
    printf(codegen)?;
    add_int(codegen)?;

    Ok(())
}

pub fn add_main_func(codegen: &mut Codegen) -> Result<()> {
    let fn_name = Ident::from("main");
    let signature = FuncSignature {
        name: fn_name.clone(),
        return_type: CrabType::UINT,
        unnamed_params: vec![],
        named_params: vec![],
    };

    let main_fn_value = codegen.get_module().get_function(&internal_main_func_name()).unwrap();

    codegen.register_function(signature.clone(), true, None)?;

    let (_, builder) = begin_func(codegen, &signature)?;
    let csv = builder.build_call(main_fn_value, &[], "call");
    let crab_value = CrabValueType::from_call_site_value(csv, CrabType::STRUCT(int_struct_name()));
    let loaded = builder.build_load(crab_value.try_as_struct_value()?, "loaded");

    let result = builder.build_extract_value(loaded.into_struct_value(), 0, "result").unwrap();
    builder.build_return(Some(&result));

    Ok(())
}

fn printf(codegen: &mut Codegen) -> Result<()> {
    let signature = FuncSignature {
        name: Ident::from("printf"),
        return_type: CrabType::FLOAT,
        unnamed_params: vec![FnParam {
            name: Ident::from("str"),
            crab_type: CrabType::STRING,
        }],
        named_params: vec![],
    };
    codegen.register_function(signature.clone(), true, Some(Linkage::External))?;
    Ok(())
}

fn add_int(codegen: &mut Codegen) -> Result<()> {
    let fn_name = Ident::from("__add_int__");
    let signature = FuncSignature {
        name: fn_name.clone(),
        return_type: CrabType::UINT,
        unnamed_params: vec![FnParam {
            name: Ident::from("lhs"),
            crab_type: CrabType::UINT,
        },
                             FnParam {
                                 name: Ident::from("rhs"),
                                 crab_type: CrabType::UINT,
                             }
        ],
        named_params: vec![],
    };

    codegen.register_function(signature.clone(), true, None)?;

    let (fn_value, builder) = begin_func(codegen, &signature)?;

    let result = builder.build_int_add(fn_value.get_nth_param(0).unwrap().into_int_value(), fn_value.get_nth_param(1).unwrap().into_int_value(), "addition");
    builder.build_return(Some(&result));

    Ok(())
}

fn begin_func<'ctx>(codegen: &'ctx mut Codegen, sig: &'ctx FuncSignature) -> Result<(FunctionValue<'ctx>, Builder<'ctx>)> {
    //TODO: Lots of unwraps here... really feeling lazy today
    let fn_value = codegen.get_module().get_function(&sig.name).unwrap();
    let basic_block = codegen.get_context().append_basic_block(fn_value, "entry");
    let builder = codegen.get_context().create_builder();
    builder.position_at_end(basic_block);
    Ok((fn_value, builder))
}
