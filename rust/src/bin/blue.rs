use anyhow::{anyhow, Result};
use crab::compile::llvmgen::{Codegen};
use crab::parse::parse;
use inkwell::context::Context;
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "blue")]
struct Args {
    /// Input file
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Skip verifying the emitted ir
    #[structopt(short, long)]
    no_verify: bool,
}

fn handle_crabfile(crabfile: PathBuf, package: &str, verify: bool) -> Result<()> {
    // parse crabfile
    info!(
        "Handling crabfile {:?} in package {:?}",
        crabfile.display(),
        package
    );
    let parse_result = parse(&crabfile)?;
    debug!("\nCrabfile handled. Results:\n");
    debug!("{:#?}", parse_result);

    // build llvm ir
    let context = Context::create();
    let module = context.create_module("main");
    let mut codegen = Codegen::new(&context, &module);
    codegen.compile(parse_result)?;

    // Use unwrap because of weird thread-safety compiler checks
    if verify {
        module.verify().unwrap();
    }
    codegen.print_to_file(PathBuf::from("out.ll")).unwrap();
    info!("Successfully wrote llvm ir to 'out.ll'");
    Ok(())
}

fn _main() -> Result<()> {
    let args = Args::from_args();

    if args.verbose == 0 {
        SimpleLogger::new()
            .with_level(LevelFilter::Info)
            .init()
            .unwrap();
    } else if args.verbose == 1 {
        SimpleLogger::new()
            .with_level(LevelFilter::Debug)
            .init()
            .unwrap();
    } else if args.verbose == 2 {
        SimpleLogger::new()
            .with_level(LevelFilter::Trace)
            .init()
            .unwrap();
    } else {
        return Err(anyhow!(
            "Invalid number of verbose flags. Expected 0-2, instead got {}",
            args.verbose
        ));
    }

    handle_crabfile(PathBuf::from(args.path), "UwU", !args.no_verify)?;

    // for package in packages {
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
    // handle_crabfile(PathBuf::from(package), "UwU")?;
    // }

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
