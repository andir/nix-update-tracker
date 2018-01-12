#![feature(slice_concat_ext)]
#[macro_use]
extern crate log;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate serde_json;

extern crate md5;
extern crate fs2;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::ffi::OsString;

#[macro_use]
extern crate serde_derive;
extern crate reqwest;

mod nvd;
mod log_config;
mod fs_cache;
mod nix;
mod nix_patches;
mod git;

use nvd::NVDSearchable;
use nix::PackageVersion;
#[derive(Serialize, Deserialize, Clone)]
struct Project {
    backend: String,
    created_on: f64,
    homepage: String,
    id: i64,
    name: String,
    //	packages: Vec<struct {
    //		distro: String
    //		package_name: String
    //	}>,
    updated_on: f64,
    version: Option<String>,
    //version_url: Option<String>, // sometimes null
    versions: Vec<String>,
}

fn fix_version(pkgname: String, version: String) -> String {
    if (version.starts_with('v') || version.starts_with('-')) && version.len() > 1 {
        let strippedVersion: String = String::from(&version[1..]);
        fix_version(pkgname, strippedVersion)
    } else if version.starts_with(&pkgname) {
        let strippedVersion: String = String::from(&version[pkgname.len()..]);
        fix_version(pkgname, strippedVersion)
    } else {
        version
    }
}

fn get_tracker_packages(update: bool) -> std::result::Result<HashMap<String, Project>, String> {
    #[derive(Serialize, Deserialize)]
    struct ProjectListJSON {
        projects: std::vec::Vec<Project>,
        total: i64,
    };

    let url = "https://release-monitoring.org/api/projects";
    let mut results = match fs_cache::http_get_cached("projects.json", url, update) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    let content = match std::str::from_utf8(&results.as_slice()) {
        Ok(c) => c,
        Err(e) => return Err(format!("failed to read response: {:?}", e)),
    };

    let mut results: ProjectListJSON = match serde_json::from_str(&content) {
        Ok(o) => o,
        Err(e) => {
            println!("{}", url);
            return Err(format!("Failed to deserialize json: {:?}", e));
        }
    };

    let mut ret: HashMap<String, Project> = HashMap::new();
    for project in results.projects {
        let mut p = project.clone();
        p.version = match p.version {
            Some(v) => Some(fix_version(p.name.clone(), v)),
            None => p.version,
        };
        ret.insert(project.name.clone(), p);
    }
    Ok(ret)
}



fn main() {
    log_config::configure(log::LogLevelFilter::Info);
    info!(crate_version!());
    let matches = clap_app!(myapp =>
	    (version: crate_version!())
	    (author: crate_authors!())
	    (@arg update: -u --update "Update. Don't use cached data.")
	    (@arg SOURCE: -s --source +required +takes_value "nixpkgs source branch.")
	    (@arg OUTPUT: -o --output +required +takes_value "output file")
    ).get_matches();

    let update = matches.is_present("update");
    let source_branch = matches.value_of("SOURCE").unwrap().to_owned();
    let output = matches.value_of("OUTPUT").unwrap();

    let cached_cloner = git::checkout::cached_cloner(Path::new("nixpkgs"));
    let nixpkgs = cached_cloner.project("nixpkgs".to_owned(), "git://github.com/nixos/nixpkgs".to_owned());

    // FIXME: should we use some other value for id?
    let nixpkgs_co = nixpkgs.clone_for(source_branch.clone(), source_branch.clone()).unwrap();
    let source = nixpkgs_co.checkout_origin_ref(&OsString::from(source_branch)).unwrap();

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching package data from release-monitoring.org");
    let trackedPackages = get_tracker_packages(update).unwrap();
    info!("Fetching the local nixos packages");
    let nixosPackages = nix::get_nixos_packages(&source, update).unwrap();
    info!("Fetching NVD CVE database");
    let vulns = nvd::get_nvd_databases(2012, update).unwrap();
    info!("Pre-processing the NVD CVE database");
 //   let prevulns = nvd::pre_process_nvd_database(&vulns);

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
    };

    let mut diff: HashMap<String, Diff> = HashMap::new();

    for (name, package) in &nixosPackages {
        let pkgname = package.pkgname();
        let version = package.version();
//        let vulns: Vec<&nvd::Cve> = prevulns.get(&pkgname).map_or(vec![], |p: &HashMap<String, Vec<&nvd::Cve>>| {
//            p.get(&version).map_or_else(|| vec![], |v| (*v).clone())
//        });

        let vulns: Vec<nvd::SimpleCVEItem> = vulns.get(&pkgname, &version);
        let trackedPackage: Option<&Project> = trackedPackages.get(&pkgname);

        let trackedNewer = (&trackedPackage).map_or(false, |v| {
            v.clone().version.map_or(false, |v| &version < &v)
        });

        if trackedNewer || vulns.len() > 0 {
            let trackerVersion = (&trackedPackage).map_or_else(|| "N/A".to_owned(), |v| {
                v.clone().version.unwrap_or("N/A".to_owned())
            });
            let homepage = (&trackedPackage).map_or_else(|| "".to_owned(), |v| v.homepage.clone());
            let maintainers = package.meta.maintainers.clone();

            let nixMaintainers =
                nix::format_maintainers(maintainers.unwrap_or(nix::NixOSPackageMaintainers::Many(vec![])));
            let patches = nix_patches::get_patches(&name, &source).unwrap();
            println!("patches: {} {:?}", pkgname, patches);
            let mut d = Diff {
                attributeName: name.clone(),
                packageName: pkgname,
                version: version,
                upstreamVersion: trackerVersion,
                maintainers: nixMaintainers,
                website: homepage,
                cves: vulns.clone(),
                patches: patches.clone(),
            };
            diff.insert(name.clone(), d);
        }
    }
    let json = match serde_json::to_string(&diff) {
        Ok(s) => s,
        Err(e) => {
            println!("failed to serialize data: {:?}", e);
            std::process::exit(1)
        }
    };
    let mut fh = match File::create(output) {
        Ok(fh) => fh,
        Err(e) => {
            println!("failed to create data.json: {:?}", e);
            std::process::exit(1)
        }
    };
    fh.write_all(&json.as_bytes()).unwrap();
}
