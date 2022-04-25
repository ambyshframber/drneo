use std::env::set_current_dir;
use std::fs::{read_to_string, read};
use std::path::Path;
use std::collections::HashMap;

use neoercities::{NeocitiesClient, site_info::{SiteInfo, SiteItem}};
use walkdir::WalkDir;
use comrak::{markdown_to_html, ComrakOptions};

use crate::{SiteBuilderError, cfg_error, utils::*, SITE_DIR};


pub struct Processor {
    files: Vec<(Vec<u8>, String)>,
    pub info: SiteInfo,
    md_ignore: Vec<String>,
    md_prefix: String,
    md_postfix: String,
    md_replace: HashMap<String, String>,
    md_render_options: ComrakOptions
}
impl Processor {
    pub fn new() -> Result<Processor, SiteBuilderError> {
        let mut options = Options::get()?;

        set_current_dir(&options.data_dir)?;

        let key = read_to_string("cfg/api_key").ok().ok_or(cfg_error("missing api key config"))?;
        let client = NeocitiesClient::new_with_key(&key);
        let info = SiteInfo::new(client)?;

        let md_prefix = read_to_string("cfg/md_prefix").ok().ok_or(cfg_error("missing markdown prefix config"))?;
        let md_postfix = read_to_string("cfg/md_postfix").ok().ok_or(cfg_error("missing markdown postfix config"))?;
        
        let mut md_ignore = Vec::new();
        match open_or_none("cfg/md_ignore")? {
            Some(s) => {
                for file in s.split('\n') {
                    md_ignore.push(String::from(file))
                }
            }
            None => {}
        }
        md_ignore.append(&mut options.md_ignore);

        let mut md_replace = HashMap::new();
        match open_or_none("cfg/md_replace")? {
            Some(s) => {
                for rep in s.split('\n') {
                    options.md_replace.push(String::from(rep))
                }
            }
            None => {}
        }
        for r in options.md_replace {
            let (trigger, replace) = match r.split_once('=') {
                Some(a) => a,
                None => {
                    return Err(SiteBuilderError::CfgError(format!("malformed replacement {}", r)))
                }
            };
            let trigger = format!("REP={}", trigger);
            md_replace.insert(trigger, String::from(replace));
        }

        let mut md_render_options = ComrakOptions::default();
        md_render_options.render.unsafe_ = true;

        Ok(Processor {
            files: Vec::new(),
            info,
            md_prefix, md_postfix, md_ignore,
            md_replace, md_render_options
        })
    }

    pub fn load_files(&mut self) -> Result<(), SiteBuilderError> {
        let dir = WalkDir::new(SITE_DIR);
        for entry in dir {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue
            }
            let p = entry.path();
            let path_string = p.to_str().ok_or(SiteBuilderError::PathError(p.to_string_lossy().to_string()))?;
            println!("found {}", path_string);
            if let Some(ext) = p.extension() {
                if ext.to_str().unwrap() == "md" {
                    if !self.md_ignore.iter().any(|path| path == path_string) {
                        println!("loading markdown {}", path_string);
                        self.load_markdown(p)?;
                        continue
                    }
                }
            }
            println!("loading {}", path_string);
            self.load(p)?
        }

        Ok(())
    }
    fn load(&mut self, path: &Path) -> Result<(), SiteBuilderError> {
        let file = read(path)?;
        let site_path = get_remote_path(path)?;
        self.files.push((file, site_path));        

        Ok(())
    }
    fn load_markdown(&mut self, path: &Path) -> Result<(), SiteBuilderError> {
        let file = read_to_string(path)?;
        let site_path = get_remote_path(path)?;
        let site_path = &site_path[..site_path.len() - 3];
        let site_path = format!("{}.html", site_path);

        let f_split = file.split('\n');
        let mut extra_head = Vec::new();
        let mut head_len = 0;
        for line in f_split {
            if line.starts_with("(HEAD)") {
                extra_head.push(&line[6..]);
                head_len += 6;
            }
            else if line.trim() != "" {
                break
            }
        }
        let extra_head = glue_vec_with(&extra_head, '\n');
        let prefix = self.md_prefix.replace("##EXTRAHEAD##", &extra_head);

        let mut file = String::from(&file[extra_head.len() + head_len..]);
        for (trig, rep) in &self.md_replace {
            file = file.replace(trig, &rep);
        }

        let html = markdown_to_html(&file, &self.md_render_options);

        let processed_file = format!("{}{}{}", prefix, html, self.md_postfix);
        //println!("{}", processed_file);
        let bytes = processed_file.bytes().collect();

        self.files.push((bytes, site_path));

        Ok(())
    }

    pub fn upload(&mut self) -> Result<(), SiteBuilderError> {
        let mut to_upload = Vec::new();
        for (i, (b, path)) in self.files.iter().enumerate() {
            if self.info.bytes_changed(b, &path) {
                println!("file {} changed, uploading", path);
                to_upload.push(i)
            }
        }

        let mut files = Vec::new();
        for i in to_upload {
            files.push(self.files[i].clone());
        }

        self.info.client.upload_bytes_multiple(files)?;

        Ok(())
    }

    pub fn delete_orphaned(&mut self) -> Result<(), SiteBuilderError> {
        self.info.refresh()?;

        let mut to_delete = Vec::new();
        for i in &self.info.items {
            match i {
                SiteItem::Dir(_) => continue,
                SiteItem::File(f) => {
                    if !self.files.iter().any(|file| file.1 == f.path) {
                        to_delete.push(f.path.as_str());
                        println!("deleting {}", f.path)
                    }
                }
            }
        }

        self.info.client.delete_multiple(to_delete)?;

        Ok(())
    }
}
