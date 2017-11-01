#[macro_use]
extern crate clap;
//extern crate serde;
extern crate serde_json;

use std::process::Command;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

#[macro_use]
extern crate serde_derive;

extern crate reqwest;
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
    let update_fn = |url| -> Result<Vec<u8>, String> {
        let mut results: Vec<u8> = Vec::new();
        match reqwest::get(url) {
            Ok(r) => r,
            Err(e) => return Err(format!("http request failed: {:?}", e)),
        }.read_to_end(&mut results);
        write_cache("projects.json", &results);
        Ok(results)
    };

    let mut results: Vec<u8> = {
        if !update {
            match get_cached_file("projects.json") {
                Some(r) => r,
                None => update_fn(url)?,
            }
        } else {
            update_fn(url)?
        }
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

#[derive(Serialize, Deserialize)]
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

fn get_cached_file(filename: &str) -> Option<Vec<u8>> {
    let file = match File::open(filename) {
        Ok(f) => f,
        _ => return None,
    };

    let mut buf_reader = BufReader::new(file);

    let mut contents: Vec<u8> = Vec::new();

    buf_reader.read_to_end(&mut contents).unwrap();

    Some(contents)
}

fn write_cache(filename: &str, contents: &Vec<u8>) {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        _ => {
            println!("Failed to write cache");
            return;
        }
    };

    file.write_all(&contents).unwrap();
}


fn get_nixos_packages(update: bool) -> std::result::Result<HashMap<String, NixOSPackage>, String> {

    let filename = "nix-pkgs.json";
    let update_fn = |filename| -> Result<Vec<u8>, String> {
        let output = match Command::new("nix-env")
            .args(&["-qa", "--json", "-I", "nixpkgs=/home/andi/dev/nixpkgs/"])
            .output() {
            Ok(o) => o,
            Err(e) => return Err(format!("Failed to execute commend {:}", e)),
        };

        let stdout = output.stdout;

        write_cache(filename, &stdout);
        Ok(stdout)
    };

    let stdout: Vec<u8> = {
        if !update {
            match get_cached_file(filename) {
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


fn format_maintainers(maintainers: &NixOSPackageMaintainers) -> String {
    match maintainers {
        &NixOSPackageMaintainers::One(ref maintainer) => format!("@{}", maintainer),
        &NixOSPackageMaintainers::Many(ref maintainers) => {
            let ms: Vec<String> = maintainers.into_iter().map(|m| format!("@{}", m)).collect();
            ms.join(",")
        }
    }
}

fn main() {
    let _matches = clap::App::new("nix-update-checker")
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    let trackedPackages = match get_tracker_packages(false) {
        Ok(list) => list,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1)
        }
    };

    let nixosPackages = get_nixos_packages(false).unwrap();

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct Diff {
        NixName: String,
        PkgName: String,
        NixVersion: String,
        TrackerVersion: String,
        NixMaintainers: String,
        Homepage: String,
    };

    let mut diff: HashMap<String, Diff> = HashMap::new();

    for (name, package) in &nixosPackages {
        let pkgname = package.pkgname();

        if trackedPackages.contains_key(&pkgname) {
            let version = package.version();
            let tp = &trackedPackages[&pkgname];
            match tp.version {
                Some(ref v) => {
                    if &version < v {
                        let maintainers: String = match package.meta.maintainers {
                            Some(ref maintainers) => format_maintainers(maintainers),
                            None => "".to_owned(),
                        };
                        println!(
                            "{}: {} ({}) {} -> {} {}",
                            name,
                            package.pkgname(),
                            package.name,
                            version,
                            v,
                            tp.homepage
                        );
                        diff.insert(
                            name.clone(),
                            Diff {
                                NixName: name.clone(),
                                PkgName: package.pkgname(),
                                NixVersion: version,
                                TrackerVersion: v.clone(),
                                NixMaintainers: maintainers,
                                Homepage: tp.homepage.clone(),
                            },
                        );
                    }
                }
                None => {}
            }
        }
    }
    let json = match serde_json::to_string(&diff) {
        Ok(s) => s,
        Err(e) => {
            println!("failed to serialize data: {:?}", e);
            std::process::exit(1)
        }
    };
    let mut fh = match File::create("data.json") {
        Ok(fh) => fh,
        Err(e) => {
            println!("failed to create data.json: {:?}", e);
            std::process::exit(1)
        }
    };
    fh.write_all(&json.as_bytes());
}
