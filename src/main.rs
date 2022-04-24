use std::io;
use std::process::exit;
use std::error::Error;

use thiserror::Error;
use neoercities;

use processor::Processor;

mod processor;
mod utils;

const SITE_DIR: &'static str = "site";
const SITE_DIR_LEN: usize = SITE_DIR.len();

fn main() {
    match run() {
        Ok(_) => exit(0),
        Err(e) => {
            println!("{}", e);
            match e {
                SiteBuilderError::EarlyExit => exit(0),
                SiteBuilderError::CfgError(cfg) => println!("{}", cfg),
                SiteBuilderError::PathError(path) => println!("{}", path),
                SiteBuilderError::ArgumentError => exit(2),
                _ => println!("{}", e.source().unwrap())
            }
            exit(1)
        }
    }
}

fn run() -> Result<(), SiteBuilderError> {
    let mut p = Processor::new()?;
    p.load_files()?;
    p.upload()?;
    p.delete_orphaned()?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum SiteBuilderError {
    #[error("missing config error")]
    CfgError(String),
    #[error("neocities API error")]
    NeocitiesError(#[from] neoercities::NeocitiesError),
    #[error("fs error")]
    IoError(#[from] io::Error),
    #[error("wakdir error")]
    WalkError(#[from] walkdir::Error),
    #[error("path contained invalid unicode")]
    PathError(String),
    #[error("argument error")]
    ArgumentError,
    #[error("not an error")]
    EarlyExit
}
pub fn cfg_error(err: &str) -> SiteBuilderError {
    SiteBuilderError::CfgError(String::from(err))
}
