use anyhow::Result;
use clap::{load_yaml, App};
use crab::parse::parse;
use glob::glob;
use log::{error, info, trace, warn};
use simple_logger::SimpleLogger;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;

//TODO: Don't hardcode these
const PROJECT_DIR: &str = "/mnt/raid1/shared/homeschoolmafia/projects/crab";
const INSERT_FUNC_PY: &str = "python/insert_func.py";
const PYTHON_BIN: &str = "/mnt/raid1/shared/homeschoolmafia/projects/pyenv/crab_env/bin/python";

fn handle_crabfile(crabfile: PathBuf, package: &str) -> Result<()> {
    info!("Handling crabfile {:?}", crabfile.display());
    let parse_result = parse(&crabfile)?;
    info!("\nCrabfile handled. Results:\n");
    info!("{:#?}", parse_result);

    for function in parse_result.functions {
        let sig_json = serde_json::to_string(&function.signature)?;
        trace!("Executing insert_func.py with signature {}", sig_json);
        Command::new(PYTHON_BIN)
            .arg(format!("{}/{}", PROJECT_DIR, INSERT_FUNC_PY))
            .arg(package)
            .arg(crabfile.clone())
            .arg(sig_json)
            .output()?;

        //TODO: use cmd.spawn() to run concurrently and check status en masse
    }
    Ok(())
}

fn _main() -> Result<()> {
    let yaml = load_yaml!("blue.yml");
    let app = App::from_yaml(yaml).version(env!("CARGO_PKG_VERSION"));
    let matches = app.get_matches();

    SimpleLogger::new().init().unwrap();

    let packages = matches.values_of("path").unwrap();

    for package in packages {
        for crabfile_result in
            glob(&format!("{}/src/lib/**/*.crab", package)).expect("Failed to read glob pattern")
        {
            match crabfile_result {
                Ok(path) => handle_crabfile(path, package)?,
                Err(err) => warn!("Skipping crabfile due to error: {}", err),
            }
        }
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
