#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate serde_json;

use std::collections::HashMap;
use std::collections::hash_map::Entry::Occupied;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;

#[macro_use]
extern crate serde_derive;

extern crate reqwest;

mod nvd;
mod log_config;
mod fs_cache;

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

#[derive(Serialize, Deserialize)]
struct NixOSPackageLicense {
    fullName: Option<String>,
    shortName: Option<String>,
    spdxId: Option<String>,
    url: Option<String>,
    free: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum NixOSPackageLicenseField {
    String(String),
    Strings(Vec<String>),
    One(NixOSPackageLicense),
    Many(Vec<NixOSPackageLicenseField>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum NixOSPackageMaintainers {
    One(String),
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum NixOSHomepage {
    One(String),
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize)]
struct NixOSPackageMeta {
    description: Option<String>,
    homepage: Option<NixOSHomepage>,
    license: Option<NixOSPackageLicenseField>,
    longDescription: Option<String>,
    maintainers: Option<NixOSPackageMaintainers>,
    outputsToInstall: Option<Vec<String>>,
    platforms: Option<Vec<String>>,
    position: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct NixOSPackage {
    name: String,
    system: String,
    meta: NixOSPackageMeta,
}

pub trait PackageVersion {
    fn pkgname(&self) -> String;
    fn version(&self) -> String;
}

impl PackageVersion for NixOSPackage {
    fn pkgname(&self) -> String {
        let v: Vec<&str> = self.name.rsplitn(2, '-').collect();
        if v.len() < 2 {
            self.name.clone()
        } else {
            String::from(v[1])
        }
    }
    fn version(&self) -> String {
        let v: Vec<&str> = self.name.rsplitn(2, '-').collect();
        if v.len() < 2 {
            String::from("")
        } else {
            String::from(v[0])
        }
    }
}





fn get_nixos_packages(source: &str, update: bool) -> std::result::Result<HashMap<String, NixOSPackage>, String> {
    let filename = "nix-pkgs.json";
   
    let update_fn = |filename| -> Result<Vec<u8>, String> {
        let output = match Command::new("nix-env")
            .args(
                &[
                    "--verbose",
                    "-f",
                    "<nixpkgs>",
                    "-I",
                    &format!("nixpkgs={}", source),
                    "-qa",
                    "--json",
                ],
            )
            .output() {
            Ok(o) => o,
            Err(e) => return Err(format!("Failed to execute commend {:}", e)),
        };
        println!(
            "stderr: {}",
            std::str::from_utf8(&output.stderr.as_slice()).unwrap()
        );
        let stdout = output.stdout;

        fs_cache::write_cache(filename, &stdout);
        Ok(stdout)
    };

    let stdout: Vec<u8> = {
        if !update {
            match fs_cache::read_cache(filename) {
                Some(s) => s,
                None => update_fn(filename)?,
            }
        } else {
            update_fn(filename)?
        }
    };

    let stdout: &str = match std::str::from_utf8(&stdout.as_slice()) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to convert Vec<u8> to &str: {:}", e)),
    };

    let packages: HashMap<String, NixOSPackage> = match serde_json::from_str(&stdout) {
        Ok(o) => o,
        Err(e) => {
            //	println!("Data: {}", &stdout);
            return Err(format!("Failed to deserialize json: {}", e));
        }
    };

    Ok(packages)
}


fn format_maintainers(maintainers: NixOSPackageMaintainers) -> String {
    match maintainers {
        NixOSPackageMaintainers::One(maintainer) => format!("@{}", maintainer),
        NixOSPackageMaintainers::Many(maintainers) => {
            let ms: Vec<String> = maintainers.into_iter().map(|m| format!("@{}", m)).collect();
            ms.join(",")
        }
    }
}

fn main() {
    log_config::configure(log::LogLevelFilter::Info);
    info!(crate_version!());
    let matches = clap_app!(myapp =>
	    (version: crate_version!())
	    (author: crate_authors!())
	    (@arg update: -u --update "Update. Don't use cached data.")
	    (@arg SOURCE: -s --source +required +takes_value "nixpkgs source url/location.")
	    (@arg OUTPUT: -o --output +required +takes_value "output file")
    ).get_matches();

    let update = matches.is_present("update");
    let source = matches.value_of("SOURCE").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();

    if update {
        info!("Requested update…");
    } else {
        info!("Using cached data only. If no local data is available it will still be fetched.");
    }

    info!("Fetching package data from release-monitoring.org");
    let trackedPackages = get_tracker_packages(update).unwrap();
    info!("Fetching the local nixos packages");
    let nixosPackages = get_nixos_packages(source, update).unwrap();
    info!("Fetching NVD CVE database");
    let vulns = nvd::get_nvd_database(2017, update).unwrap();
    info!("Pre-processing the NVD CVE database");
    let prevulns = nvd::pre_process_nvd_database(&vulns);

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct Diff {
        NixName: String,
        PkgName: String,
        NixVersion: String,
        TrackerVersion: String,
        NixMaintainers: String,
        Homepage: String,
	CVEs: Vec<String>,
    };

    let mut diff: HashMap<String, Diff> = HashMap::new();

    for (name, package) in &nixosPackages {
        let pkgname = package.pkgname();
        let version = package.version();
        let vulns: Vec<String> = prevulns.get(&pkgname).map_or(vec![], |p| p.get(&version).map_or(vec![], |v| v.to_vec() ));

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
          
	    let nixMaintainers = format_maintainers(maintainers.unwrap_or(NixOSPackageMaintainers::Many(vec![])));
            let mut d = Diff {
                NixName: name.clone(),
                PkgName: pkgname,
                NixVersion: version,
                TrackerVersion: trackerVersion,
                NixMaintainers: nixMaintainers,
                Homepage: homepage,
		CVEs: vulns,
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
    fh.write_all(&json.as_bytes());
}
