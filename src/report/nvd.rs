extern crate libflate;
extern crate serde_json;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Utc;

use super::fs_cache;
use std;
use std::collections::{HashMap, HashSet};
use std::io::Read;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Affects {
    vendor: Vendor,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV2 {
    #[serde(rename = "cvssV2")]
    cvss_v2: CvssV2,
    severity: String,
    #[serde(rename = "exploitabilityScore")]
    exploitability_score: f64,
    #[serde(rename = "impactScore")]
    impact_score: f64,
    #[serde(rename = "obtainAllPrivilege")]
    obtain_all_privilege: bool,
    #[serde(rename = "obtainUserPrivilege")]
    obtain_user_privilege: bool,
    #[serde(rename = "obtainOtherPrivilege")]
    obtain_other_privilege: bool,
    #[serde(rename = "userInteractionRequired")]
    user_interaction_required: Option<bool>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV3 {
    #[serde(rename = "cvssV3")]
    cvss_v3: CvssV3,
    #[serde(rename = "exploitabilityScore")]
    exploitability_score: f64,
    #[serde(rename = "impactScore")]
    impact_score: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Configurations {
    #[serde(rename = "CVE_data_version")]
    cve_data_version: String,
    nodes: Vec<Nodes>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cpe {
    vulnerable: bool,
    #[serde(rename = "previousVersions")]
    previous_versions: Option<bool>,
    #[serde(rename = "cpeMatchString")]
    cpe_match_string: Option<String>,
    #[serde(rename = "cpe23Uri")]
    cpe23_uri: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cve {
    data_type: String,
    data_format: String,
    data_version: String,
    #[serde(rename = "CVE_data_meta")]
    cve_data_meta: CveDataMeta,
    affects: Affects,
    problemtype: Problemtype,
    references: References,
    description: Description1,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveDataMeta {
    #[serde(rename = "ID")]
    id: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveItems {
    cve: Cve,
    configurations: Configurations,
    impact: Impact,
    #[serde(rename = "publishedDate")]
    published_date: String,
    #[serde(rename = "lastModifiedDate")]
    last_modified_date: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV2 {
    #[serde(rename = "vectorString")]
    vector_string: String,
    #[serde(rename = "accessVector")]
    access_vector: String,
    #[serde(rename = "accessComplexity")]
    access_complexity: String,
    authentication: String,
    #[serde(rename = "confidentialityImpact")]
    confidentiality_impact: String,
    #[serde(rename = "integrityImpact")]
    integrity_impact: String,
    #[serde(rename = "availabilityImpact")]
    availability_impact: String,
    #[serde(rename = "baseScore")]
    base_score: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV3 {
    #[serde(rename = "vectorString")]
    vector_string: String,
    #[serde(rename = "attackVector")]
    attack_vector: String,
    #[serde(rename = "attackComplexity")]
    attack_complexity: String,
    #[serde(rename = "privilegesRequired")]
    privileges_required: String,
    #[serde(rename = "userInteraction")]
    user_interaction: String,
    scope: String,
    #[serde(rename = "confidentialityImpact")]
    confidentiality_impact: String,
    #[serde(rename = "integrityImpact")]
    integrity_impact: String,
    #[serde(rename = "availabilityImpact")]
    availability_impact: String,
    #[serde(rename = "baseScore")]
    base_score: f64,
    #[serde(rename = "baseSeverity")]
    base_severity: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Description {
    lang: String,
    value: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Description1 {
    description_data: Vec<Description>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Impact {
    #[serde(rename = "baseMetricV3")]
    base_metric_v3: Option<BaseMetricV3>,
    #[serde(rename = "baseMetricV2")]
    base_metric_v2: Option<BaseMetricV2>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Nodes {
    operator: String,
    negate: Option<bool>,
    children: Option<Vec<Nodes>>,
    cpe: Option<Vec<Cpe>>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Problemtype {
    problemtype_data: Vec<ProblemtypeData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ProblemtypeData {
    description: Vec<Description>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Product {
    product_data: Vec<ProductData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ProductData {
    product_name: String,
    version: Version,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ReferenceData {
    url: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct References {
    reference_data: Vec<ReferenceData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NVD {
    #[serde(rename = "CVE_data_type")]
    cve_data_type: String,
    #[serde(rename = "CVE_data_format")]
    cve_data_format: String,
    #[serde(rename = "CVE_data_version")]
    cve_data_version: String,
    #[serde(rename = "CVE_data_numberOfCVEs")]
    cve_data_number_of_cves: String,
    #[serde(rename = "CVE_data_timestamp")]
    cve_data_timestamp: String,
    #[serde(rename = "CVE_Items")]
    cve_items: Vec<CveItems>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Vendor {
    vendor_data: Vec<VendorData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VendorData {
    vendor_name: String,
    product: Product,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Version {
    version_data: Vec<VersionData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VersionData {
    version_value: String,
}

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
                            if version_data.version_value == version {
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
                    let set = map.entry(name.clone()).or_insert_with(|| HashSet::new());
                    set.insert(name.clone());
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
                } else {
                    if let Some(set) = map.get_mut(pkg) {
                        // FIXME: isn't there an inplace motification to generate a union?
                        for cve in cves.iter() {
                            set.insert(cve.clone());
                        }
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
                } else {
                    if let Some(set) = map.get_mut(pkg) {
                        // FIXME: isn't there an inplace motification to generate a union?
                        for cve in cves.iter() {
                            set.insert(cve.clone());
                        }
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
