use std::env::set_current_dir;
use std::fs::{read_to_string, read, create_dir_all, write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use neoercities::{NeocitiesClient, site_info::{SiteInfo, SiteItem}};
use walkdir::WalkDir;
use comrak::{markdown_to_html, ComrakOptions};

use crate::{SiteBuilderError, cfg_error, utils::*, utils::{SITE_DIR, ALLOWED_FILE_TYPES}};

pub struct Processor {
    files: Vec<(Vec<u8>, String)>,
    info: Option<SiteInfo>,
    md_ignore: Vec<String>,
    md_prefix: String,
    md_postfix: String,
    md_replace: HashMap<String, String>,
    md_options: ComrakOptions,
    check_extensions: bool,
    local: Option<PathBuf>,
    dry_run: bool
}
impl Processor {
    pub fn new() -> Result<Processor, SiteBuilderError> {
        let mut options = Options::get()?;

        set_current_dir(&options.data_dir)?; // move into data dir
        
        let (local, info) = match options.local {
            Some(p) => {
                create_dir_all(&p)?;
                (Some(PathBuf::from(p)), None)
            }
            None => {
                let key = read_to_string("cfg/api_key").ok().ok_or(cfg_error("missing api key config"))?;
                (None, Some(SiteInfo::new(NeocitiesClient::new_with_key(&key))?))
            }
        };

        let md_prefix = read_to_string("cfg/md_prefix").ok().ok_or(cfg_error("missing markdown prefix config"))?;
        let md_postfix = read_to_string("cfg/md_postfix").ok().ok_or(cfg_error("missing markdown postfix config"))?;
        
        let mut md_ignore = Vec::new();
        match open_or_none("cfg/md_ignore")? { // ignore cfg file if it's not there
            Some(s) => {
                for file in s.split('\n') {
                    md_ignore.push(String::from(file))
                }
            }
            None => {}
        }
        md_ignore.append(&mut options.md_ignore); // shift command line ignores in

        let mut md_replace = HashMap::new();
        match open_or_none("cfg/md_replace")? {
            Some(s) => {
                for rep in s.split('\n') {
                    options.md_replace.push(String::from(rep)) // store in options vec to avoid allocing a new vec
                }
            }
            None => {}
        }
        for r in options.md_replace { // parse all replacements in one go
            let (trigger, replace) = match r.split_once('=') {
                Some(a) => a,
                None => {
                    return Err(SiteBuilderError::CfgError(format!("malformed replacement {}", r)))
                }
            };
            let trigger = format!("REP={}", trigger);
            md_replace.insert(trigger, String::from(replace));
        }

        let md_options = options.md_options;
        
        Ok(Processor {
            files: Vec::new(),
            info,
            md_prefix, md_postfix, md_ignore,
            md_replace, md_options,
            check_extensions: options.check_extensions,
            local,
            dry_run: options.dry_run
        })
    }

    pub fn build(&mut self) -> Result<(), SiteBuilderError> {
        self.load_files()?;
        match &self.local {
            None => {
                self.upload()?;
                self.delete_orphaned()?;
            }
            Some(_) => {
                self.output_local()?;
            }
        }

        Ok(())
    }

    pub fn load_files(&mut self) -> Result<(), SiteBuilderError> {
        let dir = WalkDir::new(SITE_DIR);
        for entry in dir {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue
            }
            let p = entry.path();
            let path_string = p.to_str().ok_or(SiteBuilderError::PathError(p.to_string_lossy().to_string()))?; // if theres invalid unicode then idk yell at the user
            println!("found {}", path_string);

            // this control flow is dodgy as all fuck but it works

            if let Some(ext) = p.extension() { // only check the extension if there is one
                let e_string = ext.to_str().unwrap(); // we know it's valid unicode already
                if e_string == "md" {
                    if !self.md_ignore.iter().any(|path| path == path_string) { // check if file is set to ignore
                        println!("loading markdown {}", path_string);
                        self.load_markdown(p)?;
                        continue
                    }
                }
                else if self.check_extensions && !ALLOWED_FILE_TYPES.contains(&e_string) { // error out if ext is invalid
                    return Err(SiteBuilderError::ExtensionError(String::from(path_string)))
                }
            }
            else if self.check_extensions { // if theres no extension and we're checking, error out
                return Err(SiteBuilderError::ExtensionError(String::from(path_string)))
            }
            println!("loading {}", path_string); // if its not markdown, and it has a valid ext, load normally
            self.load(p)?
        }

        Ok(())
    }
    fn load(&mut self, path: &Path) -> Result<(), SiteBuilderError> {
        let file = read(path)?; // read to bytes because fuck you
        let site_path = get_remote_path(path)?;
        self.files.push((file, site_path));        

        Ok(())
    }
    fn load_markdown(&mut self, path: &Path) -> Result<(), SiteBuilderError> {
        let file = read_to_string(path)?;
        let site_path = get_remote_path(path)?;
        let site_path = &site_path[..site_path.len() - 3]; // remove ".md"
        let site_path = format!("{}.html", site_path);

        let f_split = file.split('\n');
        let mut extra_head = Vec::new();
        let mut head_len = 0;
        for line in f_split {
            if line.starts_with("(HEAD)") { // isolate EXTRA HEAD *thunder*
                extra_head.push(&line[6..]);
                head_len += 6; // compensate for length of EXTRA HEAD *thunder*
            }
            else if line.trim() != "" { // allow empty lines in the EXTRA HEAD *thunder*
                break
            }
        }
        let extra_head = glue_vec_with(&extra_head, '\n'); // glue the EXTRA HEAD *thunder* back together
        let prefix = self.md_prefix.replace("##EXTRAHEAD##", &extra_head); // EXTRA HEAD *thunder*

        let mut file = String::from(&file[extra_head.len() + head_len..]); // chop off EXTRA HEAD *thunder*

        for (trig, rep) in &self.md_replace {
            file = file.replace(trig, rep)
        }

        let html = markdown_to_html(&file, &self.md_options);

        let processed_file = format!("{}{}{}", prefix, html, self.md_postfix);
        //println!("{}", processed_file);
        let bytes = processed_file.bytes().collect(); // convert to bytes

        self.files.push((bytes, site_path));

        Ok(())
    }

    pub fn upload(&mut self) -> Result<(), SiteBuilderError> {
        let mut to_upload = Vec::new();
        for (i, (b, path)) in self.files.iter().enumerate() {
            if self.info.as_ref().unwrap().bytes_changed(b, &path) { // check all files to see if they changed
                println!("file {} changed, uploading", path);
                to_upload.push(i)
            }
        }

        let mut files = Vec::new();
        for i in to_upload {
            files.push(self.files[i].clone()); // clone to avoid list fuckery
        }

        if !self.dry_run {
            self.info.as_ref().unwrap().client.upload_bytes_multiple(files)?;
        }

        Ok(())
    }

    pub fn delete_orphaned(&mut self) -> Result<(), SiteBuilderError> {
        self.info.as_mut().unwrap().refresh()?;

        let mut to_delete = Vec::new();
        for i in &self.info.as_ref().unwrap().items {
            match i {
                SiteItem::Dir(_) => continue, // don't delete directories. that would be silly
                SiteItem::File(f) => {
                    if !self.files.iter().any(|file| file.1 == f.path) {
                        to_delete.push(f.path.as_str());
                        println!("deleting {}", f.path)
                    }
                }
            }
        }

        if !self.dry_run {
            self.info.as_ref().unwrap().client.delete_multiple(to_delete)?;
        }

        Ok(())
    }

    fn output_local(&mut self) -> Result<(), SiteBuilderError> {
        let p = self.local.as_ref().unwrap(); // will always be Some
        for (data, path) in &self.files {
            let loc_path = p.join(Path::new(&path[1..])); // trim off backslash
            println!("writing to {}", loc_path.to_string_lossy());
            if !self.dry_run {
                write(loc_path, data)?
            }
        }
        Ok(())
    }
}
