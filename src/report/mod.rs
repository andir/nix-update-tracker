use chrono;
use rayon::prelude::*;
use serde_json;
use std;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::git;

mod fs_cache;
mod nix;
mod nix_channel;
mod nix_patches;
mod nvd;

use self::nix::PackageVersion;
use self::nvd::NVDSearchable;

error_chain! {
    foreign_links {
        ChannelError(nix_channel::Error);
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        run(true, &report_path, &pkgs_path, &settings);
    }

    Ok(())
}

pub fn run_for_revision(
    update: bool,
    output: &str,
    pkglist_filename: &str,
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

    run(update, output, &pkglist_filename, &settings)
}

pub fn run(update: bool, output: &str, pkglist_filename: &str, settings: &ReportSettings) {
    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));

    let nixpkgs = cached_cloner.project("nixpkgs".to_owned(), settings.repository.clone());

    let source_ref = settings.revision.to_owned();
    let nixpkgs_co = nixpkgs
        .clone_for("nix-vuln-scanner".to_string(), source_ref.clone())
        .expect("Failed to clone rev");
    let source = nixpkgs_co
        .checkout_ref(&OsString::from(source_ref))
        .expect("Failed to checkout rev");

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching the local nixos packages");
    let nixos_packages = nix::get_nixos_packages(&source, &pkglist_filename, update)
        .expect("Failed to retrieve nixos package information");
    info!("Fetching NVD CVE database");
    let vulns =
        nvd::get_nvd_databases(2012, update).expect("Failed to retrieve NVD database dumbs");

    let i = nixos_packages.par_iter().filter(|&(name, _)| {
        if let Some(ref filter) = settings.filter {
            name.contains(filter)
        } else {
            true
        }
    });

    info!("Filtering package list...");
    let interesting_packages: Vec<(&String, &nix::NixOSPackage)> = i
        .filter(|&(_name, package)| {
            let pkgname = package.pkgname();
            let version = package.version();

            let vuln_count = vulns.get(&pkgname, &version).len();

            debug!("{} has {} vulns", pkgname, vuln_count); //, has_update);

            vuln_count > 0
        })
        .collect();

    info!(
        "Retrieving patches for {} packages",
        interesting_packages.len()
    );
    let all_patches: HashMap<String, Vec<String>> = {
        let names: Vec<String> = interesting_packages
            .iter()
            .map(|&(name, _)| name.clone())
            .collect();
        nix_patches::get_multiple_patches(&names, &source)
    };

    info!("Found {} interesting packages.", interesting_packages.len());

    let packages: HashMap<String, Package> = interesting_packages
        .par_iter()
        .map(|(name, package)| {
            let name = *name;
            let pkgname = package.pkgname();
            let version = package.version();
            //let position = match package.meta.position {
            //    Some(ref p) => p.clone(),
            //    None => "".to_owned(),
            //};

            let vulns: Vec<nvd::SimpleCVEItem> = vulns.get(&pkgname, &version);
            let patches = match all_patches.get(name) {
                Some(p) => p.clone(),
                None => vec![],
            };
            let patched_cves = nix_patches::get_cves_from_patches(&patches);

            let cves = vulns
                .iter()
                .map(|vuln| CVE {
                    name: vuln.cve_id.clone(),
                    patched: patched_cves.contains(&vuln.cve_id),
                })
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
        .collect();

    let res = Report::from_package_map(&packages, &settings);

    let json = match serde_json::to_string(&res) {
        Ok(s) => s,
        Err(e) => {
            error!("failed to serialize data: {:?}", e);
            std::process::exit(1)
        }
    };

    let mut fh = match File::create(output) {
        Ok(fh) => fh,
        Err(e) => {
            error!("failed to create data.json: {:?}", e);
            std::process::exit(1)
        }
    };

    fh.write_all(&json.as_bytes())
        .expect("failed to write json output");

    // remove the checkout directory
    debug!("deleting directory {}", source);
    if let Err(e) = std::fs::remove_dir_all(&source) {
        error!("failed to delete directory {}: {}", source, e);
        std::process::exit(1);
    }

    debug!("deleting lockfile {}.lock", source);
    if let Err(e) = std::fs::remove_file(format!("{}.lock", source)) {
        error!("Failed to delete file {}.lock: {}", source, e);
        std::process::exit(1);
    }
}
