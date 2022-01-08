use anyhow::Result;
use clap::{load_yaml, App};
use crab::parse::parse;
use glob::glob;
use log::{error, info, trace, warn};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use std::env;
use std::fs;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};

fn _main() -> Result<()> {
    let yaml = load_yaml!("exit_code_test.yml");
    let app = App::from_yaml(yaml).version(env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();

    SimpleLogger::new().init().unwrap();

    let input_path = PathBuf::from(matches.value_of("path").unwrap());
    let input_text = fs::read_to_string(input_path)?;
    let input_text_trimmed = input_text.trim();
    let code_number: i32 = input_text_trimmed.parse()?;

    let context = Context::create();
    let module = context.create_module("exit code");
    let i32_type = context.i32_type();
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn_value = module.add_function("main", main_fn_type, None);
    let main_block = context.append_basic_block(main_fn_value, "entry");

    let builder = context.create_builder();
    builder.position_at_end(main_block);
    let code_num = i32_type.const_int(code_number as u64, true);
    builder.build_return(Some(&code_num));

    info!("module: {:?}", module.clone());
    info!("builder: {:?}", builder);

    module.print_to_file("out.ir");

    // Target::initialize_native(&InitializationConfig::default()).expect("Failed to initialize native target");
    //
    // let triple = TargetMachine::get_default_triple();
    // let cpu = TargetMachine::get_host_cpu_name().to_string();
    // let features = TargetMachine::get_host_cpu_features().to_string();
    //
    // let target = Target::from_triple(&triple).unwrap();
    // let machine = target
    //     .create_target_machine(
    //         &triple,
    //         &cpu,
    //         &features,
    //         OptimizationLevel::None,
    //         RelocMode::Default,
    //         CodeModel::Default,
    //     )
    //     .unwrap();
    //
    // // create a module and do JIT stuff
    //
    // machine.write_to_file(&module, FileType::Object, "out.asm".as_ref()).unwrap();

    Ok(())
}

fn main() {
    let result = _main();

    if let Err(error) = result {
        error!("{:?}", error);
        exit(1);
    }
}
