use anyhow::{anyhow, Result};
use crab::compile::Codegen;
use crab::parse::parse;
use glob::glob;
use inkwell::context::Context;
use log::{debug, error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "crabc")]
struct Args {
    /// Input files
    #[structopt(parse(from_os_str))]
    paths: Vec<PathBuf>,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// The name of the output file to write
    #[structopt(short, long, default_value = "out.exe")]
    output: PathBuf,

    /// Perform an optimized build
    #[structopt(long)]
    release: bool,

    /// Where to find the c builtins library, which must be linked
    #[structopt(short, long)]
    c_builtins: PathBuf,

    // Use debug_assertions to tell whether this is a debug or release build
    // If it is a debug build, add a flag to disable the verify step
    /// Skip verifying the emitted ir
    #[cfg(debug_assertions)]
    #[structopt(short, long)]
    no_verify: bool,

    // Use debug_assertions to tell whether this is a debug or release build
    // If it is a release build, add a flag to enable the verify step
    /// Verify the emitted ir
    #[cfg(not(debug_assertions))]
    #[structopt(long)]
    verify: bool,
}

fn get_crabfiles(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut crabfiles = vec![];
    for path in paths {
        if path.is_file() {
            crabfiles.push(path);
        } else if path.is_dir() {
            let source_file = path
                .join(PathBuf::from("src/**/*.crab"))
                .into_os_string()
                .into_string()
                .unwrap();
            debug!("Searching for files in {:#?}", source_file);
            for crabfile_result in glob(&source_file).expect("Failed to read glob pattern") {
                match crabfile_result {
                    Ok(crabfile) => crabfiles.push(crabfile),
                    Err(err) => warn!("Skipping crabfile due to error: {}", err),
                }
            }
        } else {
            unreachable!()
        }
    }
    crabfiles
}

fn handle_crabfile(crabfiles: &[PathBuf], verify: bool, artifact_path: &Path) -> Result<()> {
    // parse crabfile
    info!("Parsing crabfiles");
    let parse_result = parse(crabfiles)?;
    debug!("Crabfiles parsed");

    // build llvm ir
    debug!("Generating IR");
    let context = Context::create();
    let module = context.create_module("main");
    let mut codegen = Codegen::new(&context, &module);
    codegen.compile(parse_result)?;

    if verify {
        debug!("Verifying generated IR");
        // Use unwrap because of weird thread-safety compiler checks
        module.verify().unwrap();
    }
    debug!("Printing to file");
    #[cfg(debug_assertions)]
    codegen.print_to_file(artifact_path).unwrap();
    #[cfg(not(debug_assertions))]
    codegen.write_bitcode_to_file(artifact_path)?;
    info!("Successfully wrote llvm ir to 'out.ll'");
    Ok(())
}

fn clang_compile(
    artifact_path: &Path,
    output: &Path,
    c_builtins: &Path,
    release: bool,
) -> Result<()> {
    let opt_level = if release { "-O3" } else { "-O0" };
    let cmd_output = Command::new("clang")
        .arg("-fuse-ld=lld-link")
        .args([
            "-L",
            c_builtins
                .as_os_str()
                .to_str()
                .expect("Failed to stringify c_builtins"),
        ])
        .args(["-l", "crabcbuiltins"])
        .args([
            "-o",
            output
                .as_os_str()
                .to_str()
                .expect("Failed to stringify output"),
        ])
        .arg(opt_level)
        .arg(artifact_path)
        .output()?;

    info!("{}", String::from_utf8_lossy(&cmd_output.stdout));
    if !cmd_output.status.success() {
        error!("{}", String::from_utf8_lossy(&cmd_output.stderr));
        Err(anyhow!("Failed to compile {:?} with clang!", artifact_path))
    } else {
        warn!("{}", String::from_utf8_lossy(&cmd_output.stderr));
        Ok(())
    }
}

fn _main() -> Result<()> {
    let args = Args::from_args();
    let artifact_name = PathBuf::from(
        args.output
            .file_stem()
            .ok_or(anyhow!("Failed to get file stem of {:?}", args.output))?,
    );
    let target_dir: &Path = args
        .output
        .parent()
        .ok_or(anyhow!("Failed to get parent of {:?}", args.output))?;
    #[cfg(debug_assertions)]
    let artifact_path = target_dir.join(artifact_name.with_extension("ll"));
    #[cfg(not(debug_assertions))]
    let artifact_path = target_dir.join(artifact_name.with_extension("bc"));

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

    info!("Compiling {:#?}", args.paths);

    let paths = get_crabfiles(args.paths);

    // Use debug_assertions to tell whether this is a debug or release build
    // If it is a debug build, enable verify by default, but override with the no_verify flag
    // If iti s a release build, disable verify by default, but override with the verify flag
    #[cfg(debug_assertions)]
    let verify = !args.no_verify;
    #[cfg(not(debug_assertions))]
    let verify = args.verify;

    handle_crabfile(&paths, verify, &artifact_path)?;
    clang_compile(&artifact_path, &args.output, &args.c_builtins, args.release)?;

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
