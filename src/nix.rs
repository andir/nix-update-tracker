use serde_json;
use std::collections::HashMap;
use std::process::Command;
use std;
use fs_cache;

#[derive(Serialize, Deserialize)]
pub struct NixOSPackageLicense {
    fullName: Option<String>,
    shortName: Option<String>,
    spdxId: Option<String>,
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
#[serde(untagged)]
pub enum NixOSPackageMaintainers {
    One(String),
    Many(Vec<String>),
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
    pub longDescription: Option<String>,
    pub maintainers: Option<NixOSPackageMaintainers>,
    pub outputsToInstall: Option<Vec<String>>,
    pub platforms: Option<Vec<String>>,
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





pub fn get_nixos_packages(
    source: &str,
    update: bool,
) -> std::result::Result<HashMap<String, NixOSPackage>, String> {
    let filename = "nix-pkgs.json";

    let update_fn = |filename| -> Result<Vec<u8>, String> {
        let output = match Command::new("nix-env")
            .args(
                &[
                    "--verbose",
                    "-f",
                    &format!("{}", source),
                    //"<nixpkgs>",
                    //"-I",
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


pub fn format_maintainers(maintainers: NixOSPackageMaintainers) -> Vec<String> {
    match maintainers {
        NixOSPackageMaintainers::One(maintainer) => vec![format!("@{}", maintainer)],
        NixOSPackageMaintainers::Many(maintainers) => {
            let ms: Vec<String> = maintainers.into_iter().map(|m| format!("@{}", m)).collect();
            ms
        }
    }
}