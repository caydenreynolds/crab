use anyhow::Result;
use crab::parse::parse;
use glob::glob;
use log::{error, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "print_ast")]
struct Args {
    /// Input files
    #[structopt(parse(from_os_str))]
    paths: Vec<PathBuf>,
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

fn _main() -> Result<()> {
    let args = Args::from_args();

    SimpleLogger::new()
        .with_level(LevelFilter::Error)
        .init()
        .unwrap();

    let paths = get_crabfiles(args.paths);
    let parse_result = parse(&paths)?;
    print!("{:#?}", parse_result);

    Ok(())
}

fn main() {
    let result = _main();

    if let Err(error) = result {
        error!("{:?}", error);
        exit(1);
    }
}
