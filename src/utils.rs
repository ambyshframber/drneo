use std::path::Path;
use std::fmt::Display;
use std::io;
use std::fs::read_to_string;

use argparse::{ArgumentParser, Store, Collect, StoreFalse};

use crate::SiteBuilderError;

pub const SITE_DIR: &'static str = "site";
const SITE_DIR_LEN: usize = SITE_DIR.len();

pub const ALLOWED_FILE_TYPES: &[&'static str] = &["asc", "atom", "bin", "css", "csv", "dae", "eot", "epub", "geojson", "gif", "gltf", "htm", "html", "ico", "jpeg", "jpg", "js", "json", "key", "kml", "knowl", "less", "manifest", "markdown", "md", "mf", "mid", "midi", "mtl", "obj", "opml", "otf", "pdf", "pgp", "png", "rdf", "rss", "sass", "scss", "svg", "text", "tsv", "ttf", "txt", "webapp", "webmanifest", "webp", "woff", "woff2", "xcf", "xml"];

#[derive(Default)]
pub struct Options {
    pub data_dir: String,
    pub md_ignore: Vec<String>,
    pub md_replace: Vec<String>,
    pub check_extensions: bool,
}

impl Options {
    pub fn get() -> Result<Options, SiteBuilderError> {
        let mut o = Self::default();
        o.data_dir = String::from(".");
        o.check_extensions = true;

        {
            let mut ap = ArgumentParser::new();

            ap.refer(&mut o.data_dir).add_option(&["-d"], Store, "the site data directory");
            ap.refer(&mut o.md_ignore).add_option(&["-i"], Collect, "path to a markdown file that should not be processed into html");
            ap.refer(&mut o.md_replace).add_option(&["-r"], Collect, "a replacement for markdown processing");
            ap.refer(&mut o.check_extensions).add_option(&["-e"], StoreFalse, "do not check file extensions against neocities' list of allowed file types");

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
