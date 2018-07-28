use chrono;
use rayon::prelude::*;
use serde_json;
use std;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use git;
use nix;
use nix_patches;
use nvd;

use nix::PackageVersion;
use nvd::NVDSearchable;

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
    pub date_time: String,
    pub packages: HashMap<String, Package>,
    pub statistics: Statistics,
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

pub fn run(update: bool, source_branch: &str, output: &str, pkglist: &str, filter: Option<&str>) {
    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));
    let nixpkgs = cached_cloner.project(
        "nixpkgs".to_owned(),
        "git://github.com/nixos/nixpkgs".to_owned(),
    );

    // FIXME: should we use some other value for id?
    let source_branch = source_branch.to_owned();
    let nixpkgs_co = nixpkgs
        .clone_for("nix-vuln-scanner".to_string(), source_branch.clone())
        .expect("Failed to clone rev");
    let source = nixpkgs_co
        .checkout_ref(&OsString::from(source_branch))
        .expect("Failed to checkout rev");

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching the local nixos packages");
    let nixosPackages = nix::get_nixos_packages(&source, &pkglist, update)
        .expect("Failed to retrieve nixos package information");
    info!("Fetching NVD CVE database");
    let vulns =
        nvd::get_nvd_databases(2012, update).expect("Failed to retrieve NVD database dumbs");

    let mut diff: HashMap<String, Package> = HashMap::new();
    let i = nixosPackages.par_iter().filter(|&(name, _)| {
        if let Some(filter) = filter {
            name.contains(&filter)
        } else {
            true
        }
    });

    info!("Filtering package list...");
    let interestingPackages: Vec<(&String, &nix::NixOSPackage)> = i.filter(|&(name, package)| {
        let pkgname = package.pkgname();
        let version = package.version();

        let vuln_count = vulns.get(&pkgname, &version).len();

        debug!("{} has {} vulns", pkgname, vuln_count); //, has_update);

        vuln_count > 0
    }).collect();

    info!(
        "Retrieving patches for {} packages",
        interestingPackages.len()
    );
    let all_patches: HashMap<String, Vec<String>> = {
        let names = interestingPackages
            .iter()
            .map(|&(name, _)| name.clone())
            .collect();
        nix_patches::get_multiple_patches(&names, &source)
    };

    info!("Found {} interesting packages.", interestingPackages.len());

    for (name, package) in interestingPackages {
        let pkgname = package.pkgname();
        let version = package.version();
        let position = match package.meta.position {
            Some(ref p) => p.clone(),
            None => "".to_owned(),
        };

        let vulns: Vec<nvd::SimpleCVEItem> = vulns.get(&pkgname, &version);
        //let maintainers = package.meta.maintainers.clone();

        //let nixMaintainers =
        //    nix::format_maintainers(maintainers.unwrap_or(nix::NixOSPackageMaintainers::Many(vec![])));
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
            cves: cves,
            patches: patches.clone(),
        };

        diff.insert(name.clone(), p);
    }

    let mut stats = Statistics {
        cve_total_count: 0,
        cve_fixed_count: 0,
    };

    for (_, pkg) in &diff {
        stats.cve_total_count += pkg.cves.len();
        stats.cve_fixed_count += pkg.cves.iter().filter(|cve| cve.patched).count();
    }
    let res = Report {
        version: "1".to_owned(),
        repo: "nixpkgs".to_owned(),
        revision: source.clone(),
        date_time: chrono::prelude::Utc::now().to_string(),
        packages: diff,
        statistics: stats,
    };
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
}
