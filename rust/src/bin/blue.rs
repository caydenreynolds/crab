use anyhow::Result;
use clap::{load_yaml, App};
use crab::compile::llvmgen::LlvmVisitor;
use crab::compile::AstVisitor;
use crab::parse::parse;
use inkwell::context::Context;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::env;
use std::path::PathBuf;
use std::process::exit;

fn handle_crabfile(crabfile: PathBuf, package: &str) -> Result<()> {
    // parse crabfile
    info!(
        "Handling crabfile {:?} in package {:?}",
        crabfile.display(),
        package
    );
    let parse_result = parse(&crabfile)?;
    info!("\nCrabfile handled. Results:\n");
    info!("{:#?}", parse_result);

    // build llvm ir
    let context = Context::create();
    let mut visitor = LlvmVisitor::new(&context);
    visitor.visit(&parse_result);

    // Use unwrap because of weird thread-safety compiler checks
    visitor.print_to_file(PathBuf::from("out.ll")).unwrap();
    info!("Successfully wrote llvm ir to 'out.ll'");

    Ok(())
}

fn _main() -> Result<()> {
    let yaml = load_yaml!("blue.yml");
    let app = App::from_yaml(yaml).version(env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();

    SimpleLogger::new().init().unwrap();

    let packages = matches.values_of("path").unwrap();

    for package in packages {
        // let blue_path = PathBuf::from(package.clone()).join("blue.sqlite");
        // if blue_path.exists() {
        //     std::fs::remove_file(blue_path)?;
        // }
        // for crabfile_result in
        //     glob(&format!("{}/src/**/*.crab", package)).expect("Failed to read glob pattern")
        // {
        //     match crabfile_result {
        //         // Canonicalize result forces use of backslashes consistently on windows
        //         Ok(path) => handle_crabfile(path, package)?,
        //         Err(err) => warn!("Skipping crabfile due to error: {}", err),
        //     }
        // }
        handle_crabfile(PathBuf::from(package), "UwU")?;
    }

    info!("Finished!");

    Ok(())
}

fn main() {
    let result = _main();

    if let Err(error) = result {
        error!("{:?}", error);
        exit(1);
    }
}
