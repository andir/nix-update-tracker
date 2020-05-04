use chrono;
use rayon::prelude::*;
use serde_json;
use std;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use itertools::Itertools;

use super::git;

mod nix;
pub mod nix_channel;
pub mod nix_patches;

use self::nix::PackageVersion;
use self::nvd::NVDSearchable;
use super::nvd;
use std::collections::HashSet;

error_chain! {
    foreign_links {
        ChannelError(nix_channel::Error);
        IoError(std::io::Error);
        JsonError(serde_json::Error);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Statistics {
    pub cve_total_count: usize,
    pub cve_fixed_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub version: String,
    pub repo: String,
    pub revision: String,
    pub channel_name: String,
    pub date_time: String,
    pub commit_time: Option<String>,
    pub advance_time: Option<String>,
    pub packages: HashMap<String, Package>,
    pub statistics: Statistics,
    pub filter: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct CVE {
    pub name: String,
    pub patched: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub attribute_name: String,
    pub package_name: String,
    pub version: String,
    pub cves: Vec<CVE>,
    pub patches: Vec<String>,
}

pub struct ReportSettings {
    pub repository: String,
    pub revision: String,
    pub channel_name: String,
    pub commit_time: Option<String>,
    pub advance_time: Option<String>,
    pub filter: Option<String>,
}

fn filter_for_interesting_packages<'a>(
    pkgs: &'a HashMap<String, nix::NixOSPackage>,
    vulns_per_package: &HashMap<String, HashSet<String>>,
    settings: &ReportSettings,
) -> Vec<(&'a String, &'a nix::NixOSPackage)> {
    let i = pkgs.par_iter().filter(|&(name, _)| {
        settings
            .filter
            .as_ref()
            .map(|f| name.contains(f))
            .unwrap_or(true)
    });

    info!("Filtering package list...");
    let interesting_packages: Vec<(&String, &nix::NixOSPackage)> = i
        .filter(|&(_name, package)| {
            let pkgname = package.pkgname();

            // Search for all vulnerabilities for the pkgname and all of its aliases
            let names = get_package_names(&pkgname);
            let pkg_vulns: HashSet<_> = names
                .iter()
                .map(|name| {
                    let pkgs: Option<&HashSet<_>> = vulns_per_package.get(name);
                    pkgs
                })
                .filter_map(|v| v)
                .flatten()
                .collect();
            let vuln_count = pkg_vulns.len();
            debug!("{} has {} vulns: {:?}", pkgname, vuln_count, pkg_vulns);
            vuln_count > 0
        })
        .collect();

    interesting_packages
}
impl Report {
    fn from_package_map(packages: &HashMap<String, Package>, settings: &ReportSettings) -> Self {
        let mut stats = Statistics {
            cve_total_count: 0,
            cve_fixed_count: 0,
        };

        for (_, pkg) in packages.iter() {
            stats.cve_total_count += pkg.cves.len();
            stats.cve_fixed_count += pkg.cves.iter().filter(|cve| cve.patched).count();
        }
        Report {
            version: "1".to_owned(),
            repo: settings.repository.clone(),
            revision: settings.revision.clone(),
            date_time: chrono::prelude::Utc::now().to_string(),
            commit_time: settings.commit_time.clone(),
            advance_time: settings.advance_time.clone(),
            packages: packages.clone(),
            statistics: stats,
            channel_name: settings.channel_name.clone(),
            filter: settings.filter.clone(),
        }
    }

    pub fn write_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string(self)?;
        let mut fh = File::create(filename)?;
        fh.write_all(&json.as_bytes())?;

        Ok(())
    }
}

pub fn run_for_channel(channel: &str, directory: &str) -> Result<()> {
    let mut p = PathBuf::new();
    p.push(directory);

    if !p.exists() {
        std::fs::create_dir(&p).chain_err(|| "failed to create directory.")?;
    } else if p.is_file() {
        error!("directory {} is a file", directory);
        return Err("directory is a file".into());
    }

    let channel = nix_channel::Channel::from_name(&channel);

    for entry in channel.entries()? {
        let revision = entry.revision;
        let report_path = {
            let mut pb = p.clone();
            pb.push(format!("{}-{}-report.json", channel.name, revision));
            String::from(pb.to_str().chain_err(|| "failed to convert path to str")?)
        };

        let pkgs_path = {
            let mut pb = p.clone();
            pb.push(format!("{}-pkgs.json.br", revision));
            String::from(pb.to_str().chain_err(|| "failed to convert path to str")?)
        };

        let patches_path = {
            let mut pb = p.clone();
            pb.push(format!("{}-patches.json.br", revision));
            String::from(pb.to_str().chain_err(|| "failed to convert path to str")?)
        };

        info!(
            "Running for revision {}, out: {}, pkgs_out: {}",
            revision, report_path, pkgs_path
        );

        let settings = ReportSettings {
            repository: String::from("git://github.com/nixos/nixpkgs"),
            revision,
            channel_name: channel.name.clone(),
            commit_time: Some(entry.commit_date),
            advance_time: Some(entry.advance_date),
            filter: None,
        };

        run(true, &pkgs_path, Some(&patches_path), &settings)?.write_to_file(&report_path)?;
    }

    Ok(())
}

pub fn run_for_revision(
    update: bool,
    output: &str,
    pkglist_filename: &str,
    patches_filename: Option<&str>,
    revision: &str,
    filter: Option<&str>,
) {
    let settings = ReportSettings {
        repository: String::from("git://github.com/nixos/nixpkgs"),
        revision: String::from(revision),
        channel_name: String::from("local"),
        commit_time: None,
        advance_time: None,
        filter: filter.map(String::from),
    };

    run(update, &pkglist_filename, patches_filename, &settings)
        .unwrap()
        .write_to_file(output)
        .unwrap();
}

pub fn get_package_names(pkgname: &str) -> Vec<String> {
    // FIXME: move to config file
    lazy_static! {
        static ref PACKAGE_ALIASES: HashMap<String, Vec<String>> = {
            let mut map = HashMap::new();
            map.insert("linux".to_owned(), vec!["linux_kernel".to_owned()]);
            map.insert("linuxPackages".to_owned(), vec!["linux_kernel".to_owned()]);
            map.insert("apache-tomcat".to_owned(), vec!["tomcat".to_owned()]);
            map
        };
    };

    let names: Vec<String> = {
        if let Some(names) = PACKAGE_ALIASES.get(pkgname) {
            debug!("Found an alias for {}: {:?}", pkgname, names);
            let mut names = names.clone();
            names.push(pkgname.to_string());
            names
        } else {
            vec![pkgname.to_string()]
        }
    };

    names
}

pub fn run(
    update: bool,
    pkglist_filename: &str,
    patches_filename: Option<&str>,
    settings: &ReportSettings,
) -> Result<Report> {
    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));

    let nixpkgs = cached_cloner.project("nixpkgs".to_owned(), settings.repository.clone());

    let source_ref = settings.revision.to_owned();
    let nixpkgs_co = nixpkgs.clone_for("nix-vuln-scanner".to_string(), source_ref.clone())?;
    let source = nixpkgs_co.checkout_ref(&OsString::from(source_ref))?;

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching the local nixos packages");
    let nixos_packages = nix::get_nixos_packages(&source, &pkglist_filename)?;
    info!("Fetching NVD CVE database");
    let vulns =
        nvd::get_nvd_databases(2002, update).expect("Failed to retrieve NVD database dumbs");

    let vulns_per_package = vulns.get_issue_package_map();
    let interesting_packages =
        filter_for_interesting_packages(&nixos_packages, &vulns_per_package, &settings);

    let all_patches: HashMap<String, Vec<String>> = {
        let names: Vec<String> = interesting_packages
            .iter()
            .map(|&(name, _)| name.clone())
            .collect();
        nix_patches::get_multiple_patches(&names, &source, patches_filename)
    };

    info!("Found {} interesting packages.", interesting_packages.len());

    let packages: HashMap<String, Package> = interesting_packages
        .par_iter()
        .map(|(name, package)| {
            let name = *name;
            let pkgname = package.pkgname();
            let version = package.version();
            let names = get_package_names(&pkgname);

            let patches = match all_patches.get(name) {
                Some(p) => p.clone(),
                None => vec![],
            };
            let patched_cves = nix_patches::get_cves_from_patches(&patches);

            let cves = names
                .iter()
                .map(|name| vulns.get(name, &version))
                .flatten()
                .map(|vuln| CVE {
                    name: vuln.cve_id.clone(),
                    patched: patched_cves.contains(&vuln.cve_id),
                })
                .unique()
                .collect();

            let p = Package {
                attribute_name: name.clone(),
                package_name: pkgname.clone(),
                version: version.clone(),
                cves,
                patches: patches.clone(),
            };

            (name.clone(), p)
        })
        .filter(|(_, p)| !p.cves.is_empty())
        .collect();

    let res = Report::from_package_map(&packages, &settings);

    // remove the checkout directory
    debug!("deleting directory {}", source);
    std::fs::remove_dir_all(&source)?;

    debug!("deleting lockfile {}.lock", source);
    std::fs::remove_file(format!("{}.lock", source))?;

    Ok(res)
}
