extern crate threadpool;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate fs2;
extern crate md5;
extern crate num_cpus;
extern crate regex;
extern crate serde_json;
extern crate time;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[macro_use]
extern crate serde_derive;
extern crate reqwest;

mod fs_cache;
mod git;
mod log_config;
mod nix;
mod nix_patches;
mod nvd;
mod release_monitoring;

use nix::PackageVersion;
use nvd::NVDSearchable;

#[derive(Serialize, Debug, Clone)]
struct Statistics {
    cve_total_count: usize,
    cve_fixed_count: usize,
}

#[derive(Serialize, Debug, Clone)]
struct Result {
    version: String,
    repo: String,
    revision: String,
    date_time: String,
    packages: HashMap<String, Package>,
    statistics: Statistics,
}

#[derive(Serialize, Debug, Clone)]
struct CVE {
    name: String,
    patched: bool,
}

#[derive(Serialize, Debug, Clone)]
struct Package {
    attribute_name: String,
    package_name: String,
    version: String,
    cves: Vec<CVE>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Diff {
    attributeName: String,
    packageName: String,
    version: String,
    upstreamVersion: String,
    maintainers: Vec<String>,
    website: String,
    cves: Vec<nvd::SimpleCVEItem>,
    patches: Vec<String>,
    patched_cves: Vec<String>,
    position: String,
}

fn main() {
    info!(crate_version!());
    let matches = clap_app!(myapp =>
	    (version: crate_version!())
	    (author: crate_authors!())
	    (@arg update: -u --update "Update. Don't use cached data.")
	    (@arg SOURCE: -s --source +required +takes_value "nixpkgs source branch.")
	    (@arg OUTPUT: -o --output +required +takes_value "output file")
        (@arg DEBUG:  -d --debug "enable debug logging")
        (@arg FILTER: -f --filter +takes_value "filter package list by string")
    ).get_matches();

    if matches.is_present("DEBUG") {
        log_config::configure(log::LogLevelFilter::Debug);
    } else {
        log_config::configure(log::LogLevelFilter::Info);
    }

    let update = matches.is_present("update");
    let source_branch = matches
        .value_of("SOURCE")
        .expect("Missing SOURCE arg")
        .to_owned();
    let output = matches.value_of("OUTPUT").expect("Missing OUTPUT arg");

    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));
    let nixpkgs = cached_cloner.project(
        "nixpkgs".to_owned(),
        "git://github.com/nixos/nixpkgs".to_owned(),
    );

    // FIXME: should we use some other value for id?
    let nixpkgs_co = nixpkgs
        .clone_for(source_branch.clone(), source_branch.clone())
        .expect("Failed to clone rev");
    let source = nixpkgs_co
        .checkout_origin_ref(&OsString::from(source_branch))
        .expect("Failed to checkout rev");

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching the local nixos packages");
    let nixosPackages = nix::get_nixos_packages(&source, update)
        .expect("Failed to retrieve nixos package information");
    info!("Fetching NVD CVE database");
    let vulns =
        nvd::get_nvd_databases(2012, update).expect("Failed to retrieve NVD database dumbs");

    let mut diff: HashMap<String, Package> = HashMap::new();
    let i = nixosPackages.iter().filter(|&(name, _)| {
        if matches.is_present("FILTER") {
            let filter = matches
                .value_of("FILTER")
                .expect("Missing FILTER parameter");
            name.contains(filter)
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
    let res = Result {
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
}
