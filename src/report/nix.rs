use crate::fs_cache;
use serde_json;
use std;
use std::collections::HashMap;
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct NixOSPackageLicense {
    #[serde(rename = "fullName")]
    full_name: Option<String>,
    #[serde(rename = "shortName")]
    short_name: Option<String>,
    #[serde(rename = "spdxId")]
    spdx_id: Option<String>,
    url: Option<String>,
    free: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NixOSPackageLicenseField {
    String(String),
    Strings(Vec<String>),
    One(NixOSPackageLicense),
    Many(Vec<NixOSPackageLicenseField>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Maintainer {
    email: Option<String>,
    github: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum NixOSPackageMaintainers {
    One(String),
    Many(Vec<String>),
    OneMaintainer(Maintainer),
    NewMany(Vec<NixOSPackageMaintainers>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NixOSHomepage {
    One(String),
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub struct NixOSPackageMeta {
    pub description: Option<String>,
    pub homepage: Option<NixOSHomepage>,
    pub license: Option<NixOSPackageLicenseField>,
    #[serde(rename = "longDescription")]
    pub long_description: Option<String>,
    pub maintainers: Option<NixOSPackageMaintainers>,
    #[serde(rename = "outputsToInstall")]
    pub outputs_to_install: Option<Vec<String>>,
    //    pub platforms: Option<Vec<String>>,
    pub position: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NixOSPackage {
    pub name: String,
    pub system: String,
    pub meta: NixOSPackageMeta,
}

pub trait PackageVersion {
    fn pkgname(&self) -> String;
    fn version(&self) -> String;
}

impl PackageVersion for NixOSPackage {
    fn pkgname(&self) -> String {
        let v: Vec<&str> = self.name.rsplitn(2, '-').collect();
        debug!("pkg name: {}, slices: {:?}", self.name, v);
        if v.len() < 2 {
            self.name.clone()
        } else {
            String::from(v[1])
        }
    }
    fn version(&self) -> String {
        let v: Vec<&str> = self.name.rsplitn(2, '-').collect();
        debug!("pkg version: {}, slices: {:?}", self.name, v);
        if v.len() < 2 {
            String::from("")
        } else {
            // remove trailing +
            let v = match v[0].find('+') {
                Some(p) => String::from(&((&v[0])[0..p])),
                None => String::from(v[0]),
            };
            debug!("version after stripping: {}", v);
            v
        }
    }
}

pub fn get_nixos_packages(
    source: &str,
    filename: &str,
) -> std::result::Result<HashMap<String, NixOSPackage>, String> {
    let update_fn = |filename| -> Result<Vec<u8>, String> {
        let output = match Command::new("nix-env")
            .args(&[
                "--verbose",
                "-f",
                &source.to_string(),
                //"<nixpkgs>",
                //"-I",
                "-qa",
                "--json",
            ])
            .output()
        {
            Ok(o) => o,
            Err(e) => return Err(format!("Failed to execute commend {:}", e)),
        };
        debug!(
            "stderr: {}",
            std::str::from_utf8(&output.stderr.as_slice()).unwrap()
        );
        let stdout = output.stdout;

        let stdout: &str = match std::str::from_utf8(&stdout.as_slice()) {
            Ok(o) => o,
            Err(e) => return Err(format!("Failed to convert Vec<u8> to &str: {:}", e)),
        };

        let mut packages: HashMap<String, NixOSPackage> = match serde_json::from_str(&stdout) {
            Ok(o) => o,
            Err(e) => {
                return Err(format!("Failed to deserialize json: {}", e));
            }
        };

        let abs_path = match std::fs::canonicalize(source) {
            Ok(p) => p.to_str().unwrap_or("").to_owned(),
            Err(_) => "".to_owned(),
        };

        if abs_path.is_empty() {
            info!(
                "Failed to convert source path {} to canonocial path str",
                source
            );
        } else {
            let abs_source: String = {
                let path = std::path::PathBuf::from(source);
                match std::fs::canonicalize(&path) {
                    Ok(p) => p.to_str().unwrap().to_owned(),
                    Err(e) => {
                        error!("Failed to canonicalize path {}: {}", source, e);
                        "".to_owned()
                    }
                }
            };
            info!("Removing all occurences of {} in meta.position", abs_source);
            for package in packages.values_mut() {
                // postprocess the position in the output so the reference to the source does make sense
                package.meta.position = match package.meta.position {
                    Some(ref s) => {
                        let s = s.replace(&abs_source, "");
                        Some(s)
                    }
                    None => None,
                };
            }
        }
        let d = serde_json::to_vec(&packages).expect("a serializable object");

        fs_cache::write_cache(filename, &d).expect("cache write error");
        Ok(d)
    };

    let input: Vec<u8> = match fs_cache::read_cache(filename) {
        Some(s) => s,
        None => update_fn(filename)?,
    };

    let input: &str = match std::str::from_utf8(&input.as_slice()) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to convert Vec<u8> to &str: {:}", e)),
    };

    let packages: HashMap<String, NixOSPackage> = match serde_json::from_str(&input) {
        Ok(o) => o,
        Err(e) => {
            return Err(format!("Failed to deserialize json: {}", e));
        }
    };

    Ok(packages)
}

#[allow(dead_code)]
pub fn format_maintainers(maintainers: NixOSPackageMaintainers) -> Vec<String> {
    match maintainers {
        NixOSPackageMaintainers::One(maintainer) => vec![format!("@{}", maintainer)],
        NixOSPackageMaintainers::OneMaintainer(maintainer) => {
            if maintainer.name.is_some() {
                vec![maintainer.name.expect("a string to be present")]
            } else {
                vec![]
            }
        }
        NixOSPackageMaintainers::NewMany(maintainers) => {
            let ms: Vec<String> = maintainers
                .into_iter()
                .map(format_maintainers)
                .flat_map(|n| n)
                .collect();
            ms
        }
        NixOSPackageMaintainers::Many(maintainers) => {
            let ms: Vec<String> = maintainers.into_iter().map(|m| m).collect();
            ms
        }
    }
}
