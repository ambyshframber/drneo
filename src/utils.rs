use std::path::Path;
use std::fmt::Display;
use std::io;
use std::fs::read_to_string;

use argparse::{ArgumentParser, Store, Collect};

use crate::{SiteBuilderError, SITE_DIR_LEN};

#[derive(Default)]
pub struct Options {
    pub data_dir: String,
    pub md_ignore: Vec<String>,
    pub md_replace: Vec<String>
}

impl Options {
    pub fn get() -> Result<Options, SiteBuilderError> {
        let mut o = Self::default();
        o.data_dir = String::from(".");

        {
            let mut ap = ArgumentParser::new();

            ap.refer(&mut o.data_dir).add_option(&["-d"], Store, "the site data directory");
            ap.refer(&mut o.md_ignore).add_option(&["-i"], Collect, "path to a markdown file that should not be processed into html");
            ap.refer(&mut o.md_replace).add_option(&["-r"], Collect, "a replacement for markdown processing");

            match ap.parse_args() {
                Ok(_) => {},
                Err(e) => {
                    if e == 0 {
                        return Err(SiteBuilderError::EarlyExit)
                    }
                    else {
                        return Err(SiteBuilderError::ArgumentError)
                    }
                }
            }
        }

        Ok(o)
    }
}

pub fn get_remote_path(p: &Path) -> Result<String, SiteBuilderError> {
    Ok(String::from(&p.to_str().unwrap()[SITE_DIR_LEN..])) // fuck this
}

pub fn glue_vec_with(vec: &Vec<impl Display>, glue: char) -> String {
    let mut ret = String::new();
    for i in vec {
        ret.push_str(&format!("{}", i));
        ret.push(glue)
    }
    ret
}

pub fn open_or_none(p: impl AsRef<Path>) -> Result<Option<String>, io::Error> {
    match read_to_string(p) {
        Ok(s) => {
            Ok(Some(s))
        }
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                Err(e)
            }
            else {
                Ok(None)
            }
        }
    }
}
