extern crate dialog;
extern crate regex;
extern crate reqwest;

use serde_yaml;
use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Error as ioError;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir;
use walkdir::WalkDir;

mod model;

error_chain!{
    foreign_links {
        WalkDirErr(::walkdir::Error);
        IoError(ioError);
        ReqwestError(::reqwest::Error);
    }

    errors {
        DirectoryDoesNotExist(d: PathBuf) {
            description("directory did not exist")
            display("directory doest not exist: '{:?}'", d.to_str())
        }

        RegexFailed(d: String) {
            description("creating regex failed")
            display("creating regex failed: {}", d)
        }

        LoadingDirectoryFailed(e: String) {
            description("failed loading directory failed")
            display("failed to load the directory: {}", e)
        }

        EditFileDoesNotExist(d: PathBuf) {
            description("The file you wanted to edit doesn't exist.")
            display("The file {:?} does not exist", d.to_str())
        }

        NoEditorConfigured(a: String) {
            description("There was no editor configured using the EDITOR environment variable.")
            display("There was no editor configured using the EDITOR environment variable.")
        }

        FailedToExecutEditor(e: String) {
            description("Executing the editor failed. ")
            display("Executing the editor caused the following error: {}", e)
        }
    }
}

fn open_in_editor(file: &PathBuf) -> Result<()> {
    if !file.exists() {
        return Err(ErrorKind::EditFileDoesNotExist(file.clone()).into());
    }

    let editor = match std::env::var("EDITOR") {
        Ok(e) => e,
        Err(e) => {
            debug!("Failed to obtain environment variable: {}", e);
            return Err(ErrorKind::NoEditorConfigured("FIXME".into()).into());
        }
    };

    let status = {
        let mut child = match std::process::Command::new(editor).args(&[&file]).spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("failed to execute EDITOR command: {}", e);
                return Err(ErrorKind::FailedToExecutEditor(format!("error: {:}", e)).into());
            }
        };

        child.wait().chain_err(|| "Failed to wait on editor child")?
    };

    if !status.success() {
        return Err(ErrorKind::FailedToExecutEditor(format!(
            "Exit status of editor signals some problem. Got: {}",
            status
        )).into());
    }

    Ok(())
}

pub fn edit(directory: PathBuf, issue_id: &str) -> Result<()> {
    if !directory.exists() {
        error!("Directory {:?} does not exist.", directory.to_str());
        return Err(ErrorKind::DirectoryDoesNotExist(directory).into());
    }

    let current_notes = match load_directory(&directory) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to load directory: {}", e);
            return Err(ErrorKind::LoadingDirectoryFailed(format!(
                "Failed to load directory: {}",
                e
            )).into());
        }
    };

    if let Some(note) = current_notes.get(issue_id) {
        open_in_editor(&PathBuf::from(note.filename.clone()))?;
    } else {
        let note = model::Note::new(issue_id.into());
        let mut p = directory.clone();
        p.push(format!("{}.yaml", issue_id));
        let s = note.to_string();
        {
            let mut fh = File::create(&p)?;
            fh.write(s.as_ref())?;
        }

        open_in_editor(&p)?;
    }

    Ok(())
}

pub fn import(directory: PathBuf, api_url: &str) -> Result<()> {
    if !directory.exists() {
        error!("Directory {:?} does not exist.", directory.to_str());
        return Err(ErrorKind::DirectoryDoesNotExist(directory).into());
    }

    let current_notes = match load_directory(&directory) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to load directory: {}", e);
            return Err(ErrorKind::LoadingDirectoryFailed(format!(
                "Failed to load directory: {}",
                e
            )).into());
        }
    };

    let mut response = reqwest::get(api_url)?;

    #[derive(Deserialize)]
    struct Entry {
        id: i32,
        identifier: String,
        description: Option<String>,
    }

    let issues: Vec<Entry> = response.json()?;

    let edit = true;
    for issue in issues {
        if current_notes.get(&issue.identifier).is_none() {
            println!(
                "Missing identifier: {} (id: {})",
                issue.identifier, issue.id
            );
            let mut note = model::Note::new(issue.identifier.clone());
            note.description = issue.description;
            let mut p = directory.clone();

            p.push(format!("{}.yaml", issue.identifier));
            let s = note.to_string();
            {
                let mut fh = File::create(&p)?;
                fh.write(s.as_ref())?;
            }

            if edit {
                open_in_editor(&p)?;
            }
        }
    }

    Ok(())
}

pub fn list(directory: PathBuf, filter: Option<&str>) -> Result<()> {
    if !directory.exists() {
        error!("Directory {:?} does not exist.", directory.to_str());
        return Err(ErrorKind::DirectoryDoesNotExist(directory).into());
    }

    let filter = match filter {
        Some(f) => match regex::Regex::new(f) {
            Ok(r) => Some(r),
            Err(e) => {
                error!("Failed to compile regex: {}", e);
                return Err(ErrorKind::RegexFailed(format!("{}", e)).into());
            }
        },
        None => None,
    };

    let notes = match load_directory(&directory) {
        Ok(n) => n,
        Err(e) => {
            error!("Failed to load directory: {}", e);
            return Err(ErrorKind::LoadingDirectoryFailed(format!(
                "Failed to load directory: {}",
                e
            )).into());
        }
    };

    let iter = notes.iter().filter(|(_, v)| {
        if let Some(ref f) = filter {
            f.is_match(&v.note.identifier)
        } else {
            true
        }
    });

    info!("Iterating through files");
    for (_, e) in iter {
        println!("entry: {:?}", e.note);
    }
    Ok(())
}

pub struct Entry {
    note: model::Note,
    filename: PathBuf,
}

fn load_directory(directory: &PathBuf) -> Result<HashMap<String, Entry>> {
    let mut m = HashMap::new();

    let filter_dir = |e: &walkdir::DirEntry| {
        let p = e.path();
        let file_name = e.file_name().to_str();

        let ends_with = |s: Option<&str>, suffix: &str| s.map_or(false, |s| s.ends_with(suffix));

        let y = ends_with(file_name, ".yml") || ends_with(file_name, ".yaml");

        let x = e.file_type().is_file() && y;

        debug!("p: {:?}, x: {}", p, x);
        x
    };

    for file in WalkDir::new(directory)
        .into_iter()
        .filter_entry(|e| !e.path().starts_with("."))
        .filter_map(|e| e.ok())
        .filter(filter_dir)
    {
        debug!("ent: {:?}", file);
        let fh = File::open(file.path().clone()).chain_err(|| "failed to open file")?;
        let mut bf = BufReader::new(fh);
        let mut content = String::new();
        bf.read_to_string(&mut content)
            .chain_err(|| "failed to read file")?;
        debug!("content: {:?}", content);

        match serde_yaml::from_str(&content) {
            Ok(note) => {
                let note: model::Note = note;
                let entry = Entry {
                    note: note.clone(),
                    filename: PathBuf::from(file.path()),
                };
                m.insert(note.identifier, entry);
            }
            Err(e) => {
                error!("Failed to read file: {}", e);
            }
        };
    }
    Ok(m)
}

#[cfg(test)]
mod tests {

    use super::*;
    extern crate mktemp;

    #[test]
    fn test_list() {
        let d = mktemp::Temp::new_dir().unwrap();
        list(d.as_ref().to_path_buf(), None).unwrap();
    }
}
