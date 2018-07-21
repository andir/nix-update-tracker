use error_chain::ChainedError;
use git2;
use serde_json;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs as fs_unix;
use openat;

use report::{Package, Report};

error_chain! {
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error) #[cfg(unix)];
        Serde(serde_json::Error);
        Git2(git2::Error);
    }
}


fn relative_path(target: &Path, source: &Path) -> PathBuf {
    // assumption: both parts are from the same root
    // remove all path components from `source` until / or "" is found
    // prepend target with N-times ../ where N is the number of removed components in sourceA

    let mut source = source.to_path_buf();
    source.pop(); // removes last component, usually filename

    let n = source.components().count();

    let mut res = PathBuf::new();
    for i in 0..n {
        res.push("..");
    }

    debug!("n of {:?} = {}", source, n);

    res.push(target);

    res
}

struct VulnRepo {
    revision: String,
    repo: git2::Repository,
}

impl VulnRepo {
    fn new(revision: &str, repo: git2::Repository) -> Self {
        Self {
            revision: revision.to_owned(),
            repo: repo,
        }
    }

    fn package_base_path(&self, pkg: &Package) -> PathBuf {
       let mut buf = PathBuf::new();
       buf.push("evals");
       buf.push(&self.revision);
       buf.push(&pkg.attribute_name);
       buf
    }

    fn base_path(&self) -> Result<PathBuf> {
        let mut buf = self.repo.path().to_path_buf();
        if buf.file_name().unwrap() == ".git" {
            if !buf.pop() {
                bail!("failed to remove final path of git repo path.");
            }
        }
        Ok(buf)
    }

    fn create_or_update_file<S>(&self, index: &mut git2::Index, path: &Path, content: Option<S>) -> Result<()> where S: Into<String>
    {
        // recursivly create directories if needed
        // remove the first part of the repo since this will usually be the .git folder within
        // the repository. In bare checkouts it is not so there we skip it.
        let file_name = match path.file_name() {
            Some(n) => n,
            None => {
                bail!("no filename found");
            },
        };

        let mut buf = self.base_path()?;

        // push the target file paths
        buf.push(path);
        // pop the final file name
        buf.pop();

        {
            let p = buf.as_path();

            if !p.exists() {
                match content {
                    Some(_) => fs::create_dir_all(p).chain_err(|| "Failed to create directory chain")?,
                    None => return Ok(()),
                }
            }
        }

        buf.push(file_name);

        match content {
            Some(c) => {
                let c: String = c.into();
                let mut fh : fs::File = fs::File::create(buf.as_path()).chain_err(|| "Failed to create file.")?;
                fh.write(c.as_bytes());
            },
            None => {
                if buf.exists() {
                    info!("Deleting file {:?}", buf.to_str());
                    fs::remove_file(buf.as_path())?;
                }
            }
        }

        index.add_path(path)?;

        Ok(())
    }

    fn symlink(&self, index: &mut git2::Index, dest: &Path, source: &Path) -> Result<()> {

        let base_path = self.base_path()?;

        let relative_path = relative_path(&source, &dest);

        let dest_file_name = dest.file_name().unwrap();
        let dest = {
            let mut buf = base_path.clone();
            buf.push(dest);
            buf
        };

        let path = {
            let mut p = dest.clone().to_path_buf();
            p.pop();
            p
        };

        debug!("mkdir -p {:?}", path);
        fs::create_dir_all(&path)?;
        let d = openat::Dir::open(&path)?;
        d.symlink(&dest, &relative_path)?;
        index.add_path(&dest.as_path())?;
        Ok(())
    }

    pub fn add_package(&self, package: &Package) -> Result<()> {
        let mut index = self.repo.index()?;

        let base_path = self.package_base_path(&package);

        {
            let path = {
                let mut p = base_path.clone();
                p.push("name");
                p
            };
            self.create_or_update_file(&mut index, &path, Some(package.package_name.clone()));
        }

        {
            let path = {
                let mut p = base_path.clone();
                p.push("patches");
                p
            };
            let content = if package.patches.len() == 0 { None } else {
                Some(package.patches.join("\n"))
            };
            self.create_or_update_file(&mut index, path.as_path(), content)?;
        }

        {
            // iterate through the issues and create them
            let issue_base = {
                let mut p = PathBuf::new();
                p.push("issues");
                p
            };
            info!("Creating symlinks to/from issues for {} issues", package.cves.len());

            for cve in &package.cves {
                let cve_path = {
                    let mut p = issue_base.clone();
                    p.push(&cve.name);
                    p
                };

                let keep_path = {
                    let mut p =cve_path.clone();
                    p.push(".keep");
                    p
                };

                // FIXME: this isn't very efficient.. always creating the same file over and over
                // again..
                self.create_or_update_file(&mut index, keep_path.as_path(), Some(""))?;

                let ref_path = {
                    let mut p = cve_path.clone();
                    p.push("refs");
                    p.push(&self.revision);
                    p.push(&package.attribute_name);
                    p
                };

                debug!("symlink {:?} -> {:?}", ref_path, base_path);
                self.symlink(&mut index, ref_path.as_path(), base_path.as_path())?;
            }
        }

        Ok(())

    }
}


struct ReportProcessor {
    repo: VulnRepo,
    report: Report,
}

impl ReportProcessor {
    pub fn new(repo: git2::Repository, report_file: &str) -> Result<Self> {
        let report = load_report(report_file)?;

        let revision = report.revision.clone();

        Ok(ReportProcessor {
            report: report,
            repo: VulnRepo::new(&revision, repo),
        })
    }



    fn create_revision_tree(&self, revision: &str) {}

    fn run(&self) -> Result<()> {
        let revision = &self.report.revision;
        for (_, pkg) in &self.report.packages {
            self.repo.add_package(&pkg);
        }
        Ok(())
    }
}

fn get_repo_path(path: Option<&str>) -> Result<String> {
    let s = match path {
        Some(p) => p.to_owned(),
        None => {
            let path =
                env::current_dir().chain_err(|| "failed to retrieve current working directory")?;
            path.to_str()
                .chain_err(|| "failed to convert path to str")?
                .to_owned()
        }
    };
    Ok(s)
}

fn load_report(source: &str) -> Result<Report> {
    let fh = fs::File::open(source)?;
    let r = serde_json::from_reader(fh)?;

    Ok(r)
}

pub fn run(source: &str, repo_path: Option<&str>) -> Result<()> {
    let repo_path: String = get_repo_path(repo_path)?;
    info!("Processing input {} for repo {}", source, repo_path);

    let flags = git2::RepositoryOpenFlags::NO_SEARCH;
    let o: Vec<&str> = vec![];
    let repo: git2::Repository = git2::Repository::open_ext(&repo_path, flags, &o)
        .chain_err(|| "failed to open git repository")?;
    //
    //    let head = repo.head()?;
    //    let commit: git2::Commit = head.peel_to_commit().chain_err(|| "failed to get commit from HEAD")?;
    //
    //    debug!("Repository opened. Current HEAD is {}", commit.id());
    //    debug!("Commit message: {}", commit.message().unwrap_or("<empty>"));

    let r = ReportProcessor::new(repo, &source)?;
    r.run()?;

    Ok(())
}
