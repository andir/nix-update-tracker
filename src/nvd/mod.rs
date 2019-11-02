extern crate libflate;
extern crate serde_json;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Utc;

use super::fs_cache;
use std;
use std::collections::{HashMap, HashSet};
use std::io::Read;

mod data;
mod version_cmp;

pub use data::*;
pub use version_cmp::*;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SimpleCVEItem {
    pub pkg_name: String,
    pub cve_id: String,
    pub version: String,
    pub impact: Impact,
    pub description: Option<String>,
}

pub trait NVDSearchable {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem>;

    fn get_cve(&self, identifier: &str) -> Option<Cve>;

    fn get_descriptions(&self) -> HashMap<String, String>;

    fn get_issue_package_map(&self) -> HashMap<String, HashSet<String>>;
}

impl Cve {
    pub fn get_description(&self) -> Option<String> {
        match self
            .description
            .description_data
            .iter()
            .filter(|desc| desc.lang == "en")
            .take(1)
            .next()
        {
            Some(v) => Some(v.value.clone()),
            None => None,
        }
    }
}

impl NVDSearchable for NVD {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem> {
        let mut results: Vec<SimpleCVEItem> = Vec::new();
        for cve_item in &self.cve_items {
            for vendor_data in &cve_item.cve.affects.vendor.vendor_data {
                for product_data in &vendor_data.product.product_data {
                    if product_data.product_name == pkgname {
                        for version_data in &product_data.version.version_data {
                            if
                            /*version_data.version_value == version */
                            version_data.matches(version) {
                                results.push(SimpleCVEItem {
                                    pkg_name: product_data.product_name.clone(),
                                    cve_id: cve_item.cve.cve_data_meta.id.clone(),
                                    version: version_data.version_value.clone(),
                                    impact: cve_item.impact.clone(),
                                    description: cve_item.cve.get_description(),
                                });
                            }
                        }
                    }
                }
            }
        }
        results
    }

    fn get_cve(&self, identifier: &str) -> Option<Cve> {
        for cve_item in &self.cve_items {
            if cve_item.cve.cve_data_meta.id == identifier {
                return Some(cve_item.cve.clone());
            }
        }
        None
    }

    fn get_descriptions(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for cve_item in &self.cve_items {
            if let Some(description) = cve_item.cve.get_description() {
                map.entry(cve_item.cve.cve_data_meta.id.clone())
                    .or_insert(description);
            }
        }

        map
    }

    fn get_issue_package_map(&self) -> HashMap<String, HashSet<String>> {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        for cve_item in &self.cve_items {
            for vendor_data in &cve_item.cve.affects.vendor.vendor_data {
                for product_data in &vendor_data.product.product_data {
                    let name = &product_data.product_name;
                    let set = map.entry(name.clone()).or_insert_with(HashSet::new);
                    set.insert(cve_item.cve.cve_data_meta.id.clone());
                }
            }
        }

        map
    }
}

#[allow(dead_code)]
pub fn pre_process_nvd_database(nvd: &NVD) -> HashMap<String, HashMap<String, Vec<SimpleCVEItem>>> {
    let mut ret: HashMap<String, HashMap<String, Vec<SimpleCVEItem>>> = HashMap::new();
    for cve_item in &nvd.cve_items {
        for vendor_data in &cve_item.cve.affects.vendor.vendor_data {
            for product_data in &vendor_data.product.product_data {
                let pkgname = &product_data.product_name;
                let node = ret.entry(pkgname.clone()).or_insert_with(HashMap::new);
                for version_data in &product_data.version.version_data {
                    let version_node = node
                        .entry(version_data.version_value.clone())
                        .or_insert_with(Vec::new);
                    version_node.push(SimpleCVEItem {
                        pkg_name: pkgname.clone(),
                        cve_id: cve_item.cve.cve_data_meta.id.clone(),
                        version: version_data.version_value.clone(),
                        impact: cve_item.impact.clone(),
                        description: cve_item.cve.get_description(),
                    });
                }
            }
        }
    }
    ret
}

impl NVDSearchable for Vec<NVD> {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem> {
        self.iter()
            .map(|nvd| nvd.get(pkgname, version))
            .fold(vec![], |mut acc, v| {
                acc.extend(v);
                acc
            })
    }

    fn get_cve(&self, identifier: &str) -> Option<Cve> {
        for l in self.iter() {
            match l.get_cve(identifier) {
                Some(c) => return Some(c),
                _ => continue,
            }
        }
        None
    }

    fn get_descriptions(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for l in self.iter() {
            map.extend(l.get_descriptions());
        }

        map
    }

    fn get_issue_package_map(&self) -> HashMap<String, HashSet<String>> {
        let mut map = HashMap::new();
        for l in self.iter() {
            let m = l.get_issue_package_map();
            for (pkg, cves) in m.iter() {
                if !map.contains_key(pkg) {
                    map.insert(pkg.clone(), cves.clone());
                } else if let Some(set) = map.get_mut(pkg) {
                    // FIXME: isn't there an inplace motification to generate a union?
                    for cve in cves.iter() {
                        set.insert(cve.clone());
                    }
                }
            }
        }
        map
    }
}

impl NVDSearchable for &[NVD] {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem> {
        self.iter()
            .map(|nvd| nvd.get(pkgname, version))
            .fold(vec![], |mut acc, v| {
                acc.extend(v);
                acc
            })
    }

    fn get_cve(&self, identifier: &str) -> Option<Cve> {
        for l in self.iter() {
            match l.get_cve(identifier) {
                Some(c) => return Some(c),
                _ => continue,
            }
        }
        None
    }

    fn get_descriptions(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for l in self.iter() {
            map.extend(l.get_descriptions());
        }

        map
    }

    fn get_issue_package_map(&self) -> HashMap<String, HashSet<String>> {
        let mut map = HashMap::new();
        for l in self.iter() {
            let m = l.get_issue_package_map();
            for (pkg, cves) in m.iter() {
                if !map.contains_key(pkg) {
                    map.insert(pkg.clone(), cves.clone());
                } else if let Some(set) = map.get_mut(pkg) {
                    // FIXME: isn't there an inplace motification to generate a union?
                    for cve in cves.iter() {
                        set.insert(cve.clone());
                    }
                }
            }
        }
        map
    }
}

pub fn get_nvd_databases(since: i32, update: bool) -> Result<Vec<NVD>, String> {
    let local_time: DateTime<Utc> = Utc::now();
    let mut nvds: Vec<NVD> = Vec::new();
    for year in since..=local_time.date().year() {
        match get_nvd_database(year, update) {
            Ok(nvd) => nvds.push(nvd),
            Err(e) => return Err(e),
        }
    }

    Ok(nvds)
}

pub fn get_nvd_database(year: i32, update: bool) -> Result<NVD, String> {
    let url = format!(
        "https://nvd.nist.gov/feeds/json/cve/1.0/nvdcve-1.0-{}.json.gz",
        year
    );

    let data = match fs_cache::http_get_cached(&format!("nvd-{}.json.gz", year), &url, update) {
        Err(e) => return Err(e),
        Ok(o) => o,
    };

    let mut decompressed_body = Vec::new();
    let mut decoder = libflate::gzip::Decoder::new(&data[..]).unwrap();
    decoder.read_to_end(&mut decompressed_body).unwrap();

    let data = match std::str::from_utf8(&decompressed_body.as_slice()) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to convert to str: {:?}", e)),
    };

    let ret: NVD = match serde_json::from_str(&data) {
        Ok(d) => d,
        Err(e) => {
            return Err(format!("Failed to deserialize json: {:?}", e));
        }
    };

    Ok(ret)
}
