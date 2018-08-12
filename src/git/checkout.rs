use md5;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use git::clone;
use git::clone::GitClonable;

pub struct CachedCloner {
    root: PathBuf,
}

pub fn cached_cloner(path: &Path) -> CachedCloner {
    return CachedCloner { root: path.to_path_buf() };
}

pub struct CachedProject {
    root: PathBuf,
    clone_url: String,
}

pub struct CachedProjectCo {
    root: PathBuf,
    id: String,
    clone_url: String,
    local_reference: PathBuf,
}

impl CachedCloner {
    pub fn project(&self, name: String, clone_url: String) -> CachedProject {
        // <root>/repo/<hash>/clone
        // <root>/repo/<hash>/clone.lock
        // <root>/repo/<hash>/<type>/<id>
        // <root>/repo/<hash>/<type>/<id>.lock

        let mut new_root = self.root.clone();
        new_root.push("repo");
        new_root.push(format!("{:x}", md5::compute(&name)));

        return CachedProject {
            root: new_root,
            clone_url: clone_url,
        };
    }
}

impl CachedProject {
    pub fn clone_for(&self, use_category: String, id: String) -> Result<CachedProjectCo, Error> {
        self.prefetch_cache()?;

        let mut new_root = self.root.clone();
        new_root.push(use_category);

        return Ok(CachedProjectCo {
            root: new_root,
            id: id,
            clone_url: self.clone_from().clone(),
            local_reference: self.clone_to().clone(),
        });
    }

    fn prefetch_cache(&self) -> Result<PathBuf, Error> {
        fs::create_dir_all(&self.root)?;

        self.clone_repo()?;
        self.fetch_repo()?;

        return Ok(self.clone_to());
    }
}

impl CachedProjectCo {
    pub fn checkout_origin_ref(&self, git_ref: &OsStr) -> Result<String, Error> {
        let mut pref = OsString::from("origin/");
        pref.push(git_ref);

        self.checkout_ref(&pref)
    }

    pub fn checkout_ref(&self, git_ref: &OsStr) -> Result<String, Error> {
        fs::create_dir_all(&self.root)?;

        self.clone_repo()?;
        self.fetch_repo()?;
        self.clean()?;
        self.checkout(git_ref)?;

        // let build_dir = self.build_dir();

        return Ok(self.clone_to().to_str().unwrap().to_string());
    }

    pub fn fetch_pr(&self, pr_id: u64) -> Result<(), Error> {
        let mut lock = self.lock()?;

        let result = Command::new("git")
            .arg("fetch")
            .arg("origin")
            .arg(format!("+refs/pull/{}/head:pr", pr_id))
            .current_dir(self.clone_to())
            .status()?;

        lock.unlock();

        if result.success() {
            return Ok(());
        } else {
            return Err(Error::new(ErrorKind::Other, "Failed to fetch PR"));
        }
    }

    pub fn commit_exists(&self, commit: &OsStr) -> bool {
        let mut lock = self.lock().expect("Failed to lock");

        let result = Command::new("git")
            .arg("--no-pager")
            .arg("show")
            .arg(commit)
            .current_dir(self.clone_to())
            .status()
            .expect("git show <commit> failed");

        lock.unlock();

        return result.success();
    }

    pub fn merge_commit(&self, commit: &OsStr) -> Result<(), Error> {
        let mut lock = self.lock()?;

        let result = Command::new("git")
            .arg("merge")
            .arg("--no-gpg-sign")
            .arg("-m")
            .arg("Automatic merge for GrahamCOfBorg")
            .arg(commit)
            .current_dir(self.clone_to())
            .status()?;

        lock.unlock();

        if result.success() {
            return Ok(());
        } else {
            return Err(Error::new(ErrorKind::Other, "Failed to merge"));
        }
    }
}

impl clone::GitClonable for CachedProjectCo {
    fn clone_from(&self) -> String {
        return self.clone_url.clone();
    }

    fn clone_to(&self) -> PathBuf {
        let mut clone_path = self.root.clone();
        clone_path.push(&self.id);
        return clone_path;
    }

    fn lock_path(&self) -> PathBuf {
        let mut lock_path = self.root.clone();
        lock_path.push(format!("{}.lock", self.id));
        return lock_path;
    }

    fn extra_clone_args(&self) -> Vec<&OsStr> {
        let local_ref = self.local_reference.as_ref();
        return vec![
            OsStr::new("--shared"),
            OsStr::new("--reference-if-able"),
            local_ref,
        ];
    }
}

impl clone::GitClonable for CachedProject {
    fn clone_from(&self) -> String {
        return self.clone_url.clone();
    }

    fn clone_to(&self) -> PathBuf {
        let mut clone_path = self.root.clone();
        clone_path.push("clone");
        return clone_path;
    }

    fn lock_path(&self) -> PathBuf {
        let mut clone_path = self.root.clone();
        clone_path.push("clone.lock");
        return clone_path;
    }

    fn extra_clone_args(&self) -> Vec<&OsStr> {
        return vec![OsStr::new("--bare")];
    }
}
