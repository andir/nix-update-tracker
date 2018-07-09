use std::collections::HashMap;
use std::result::Result;
use std::str::from_utf8;
use std::vec::Vec;

use serde_json;

use fs_cache;

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub backend: String,
    pub created_on: f64,
    pub homepage: String,
    pub id: i64,
    pub name: String,
    //	packages: Vec<struct {
    //		distro: String
    //		package_name: String
    //	}>,
    pub updated_on: f64,
    pub version: Option<String>,
    //version_url: Option<String>, // sometimes null
    pub versions: Vec<String>,
}

impl Project {
    pub fn version_lower(&self, version: &str) -> bool {
        match &self.version {
            &Some(ref v) => {
                let _v: &str = &v;
                version < _v
            }
            &None => false,
        }
    }
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

pub fn get_tracker_packages(update: bool) -> Result<HashMap<String, Project>, String> {
    #[derive(Serialize, Deserialize)]
    struct ProjectListJSON {
        projects: Vec<Project>,
        total: i64,
    };

    let url = "https://release-monitoring.org/api/projects";
    let results = match fs_cache::http_get_cached("projects.json", url, update) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    let content = match from_utf8(&results.as_slice()) {
        Ok(c) => c,
        Err(e) => return Err(format!("failed to read response: {:?}", e)),
    };

    let results: ProjectListJSON = match serde_json::from_str(&content) {
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
