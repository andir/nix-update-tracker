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
extern crate serde_json;
extern crate time;
extern crate regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ffi::OsString;

#[macro_use]
extern crate serde_derive;
extern crate reqwest;

mod release_monitoring;
mod nvd;
mod log_config;
mod fs_cache;
mod nix;
mod nix_patches;
mod git;

use nvd::NVDSearchable;
use nix::PackageVersion;



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
    let source_branch = matches.value_of("SOURCE").expect("Missing SOURCE arg").to_owned();
    let output = matches.value_of("OUTPUT").expect("Missing OUTPUT arg");

    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));
    let nixpkgs = cached_cloner.project("nixpkgs".to_owned(), "git://github.com/nixos/nixpkgs".to_owned());

    // FIXME: should we use some other value for id?
    let nixpkgs_co = nixpkgs.clone_for(source_branch.clone(), source_branch.clone()).expect("Failed to clone rev");
    let source = nixpkgs_co.checkout_origin_ref(&OsString::from(source_branch)).expect("Failed to checkout rev");

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching package data from release-monitoring.org");
    let trackedPackages = release_monitoring::get_tracker_packages(update).expect("Failed to retrieve upstream package information");
    info!("Fetching the local nixos packages");
    let nixosPackages = nix::get_nixos_packages(&source, update).expect("Failed to retrieve nixos package information");
    info!("Fetching NVD CVE database");
    let vulns = nvd::get_nvd_databases(2012, update).expect("Failed to retrieve NVD database dumbs");

    let mut diff: HashMap<String, Diff> = HashMap::new();
    let i = nixosPackages.iter().filter(|&(name, _)| {
        if matches.is_present("FILTER") {
            let filter = matches.value_of("FILTER").expect("Missing FILTER parameter");
            name.contains(filter)
        } else {
            true
        }
    });

    let interestingPackages : Vec<(&String, &nix::NixOSPackage)> = i.filter(|&(name, package)| {
        let pkgname = package.pkgname();
        let version = package.version();

        let vuln_count = vulns.get(&pkgname, &version).len();
        let has_update = trackedPackages.get(&pkgname).map_or(false, |t| {
            t.version_lower(&version)
        });

        debug!("{} has {} vulns, and {} update", pkgname, vuln_count, has_update);

        vuln_count > 0 || has_update
    }).collect();


    let all_patches: HashMap<String, Vec<String>> = {
        let names = interestingPackages.iter().map(|&(name, _)| name.clone()).collect();
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
        let trackedPackage = trackedPackages.get(&pkgname);

        let trackerVersion = (&trackedPackage).map_or_else(|| "N/A".to_owned(), |v| {
            v.clone().version.unwrap_or("N/A".to_owned())
        });
        let homepage = (&trackedPackage).map_or_else(|| "".to_owned(), |v| v.homepage.clone());
        let maintainers = package.meta.maintainers.clone();

        let nixMaintainers =
            nix::format_maintainers(maintainers.unwrap_or(nix::NixOSPackageMaintainers::Many(vec![])));
        let patches = match all_patches.get(name) {
            Some(p) => p.clone(),
            None => vec![],
        };
        let patched_cves = nix_patches::get_cves_from_patches(&patches);

        debug!("patches: {} {:?}", pkgname, patches);
        let mut d = Diff {
            attributeName: name.clone(),
            packageName: pkgname,
            version: version,
            upstreamVersion: trackerVersion,
            maintainers: nixMaintainers,
            website: homepage,
            cves: vulns.clone(),
            patches: patches,
            patched_cves: patched_cves,
            position: position.clone(),
        };
        diff.insert(name.clone(), d);
    }
    let json = match serde_json::to_string(&diff) {
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
    fh.write_all(&json.as_bytes()).expect("failed to write json output");
}
