extern crate libflate;
extern crate serde_json;

use chrono::DateTime;
use chrono::Utc;
use chrono::Datelike;

use fs_cache;
use reqwest;
use std;
use std::io::Read;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Affects {
    vendor: Vendor,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV2 {
    cvssV2: CvssV2,
    severity: String,
    exploitabilityScore: f64,
    impactScore: f64,
    obtainAllPrivilege: bool,
    obtainUserPrivilege: bool,
    obtainOtherPrivilege: bool,
    userInteractionRequired: Option<bool>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV3 {
    cvssV3: CvssV3,
    exploitabilityScore: f64,
    impactScore: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Configurations {
    CVE_data_version: String,
    nodes: Vec<Nodes>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cpe {
    vulnerable: bool,
    previousVersions: Option<bool>,
    cpeMatchString: Option<String>,
    cpe23Uri: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cve {
    data_type: String,
    data_format: String,
    data_version: String,
    CVE_data_meta: CveDataMeta,
    affects: Affects,
    problemtype: Problemtype,
    references: References,
    description: Description1,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveDataMeta {
    ID: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveItems {
    cve: Cve,
    configurations: Configurations,
    impact: Impact,
    publishedDate: String,
    lastModifiedDate: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV2 {
    vectorString: String,
    accessVector: String,
    accessComplexity: String,
    authentication: String,
    confidentialityImpact: String,
    integrityImpact: String,
    availabilityImpact: String,
    baseScore: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV3 {
    vectorString: String,
    attackVector: String,
    attackComplexity: String,
    privilegesRequired: String,
    userInteraction: String,
    scope: String,
    confidentialityImpact: String,
    integrityImpact: String,
    availabilityImpact: String,
    baseScore: f64,
    baseSeverity: String,
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

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Impact {
    baseMetricV3: Option<BaseMetricV3>,
    baseMetricV2: Option<BaseMetricV2>,
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
    CVE_data_type: String,
    CVE_data_format: String,
    CVE_data_version: String,
    CVE_data_numberOfCVEs: String,
    CVE_data_timestamp: String,
    CVE_Items: Vec<CveItems>,
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
   pkg_name: String,
   cve_id: String,
   version: String,
}

pub trait NVDSearchable {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem>;
}

impl NVDSearchable for NVD {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem> {
        let mut results: Vec<SimpleCVEItem> = Vec::new();
        for cve_item in &self.CVE_Items {
            for vendor_data in &cve_item.cve.affects.vendor.vendor_data {
                for product_data in &vendor_data.product.product_data {
                    if product_data.product_name == pkgname {
                        for version_data in &product_data.version.version_data {
                            if version_data.version_value == version {
                //                results.push(cveItem.cve.CVE_data_meta.ID.clone())
                                results.push(SimpleCVEItem{
                                  pkg_name: product_data.product_name.clone(),
                                  cve_id: cve_item.cve.CVE_data_meta.ID.clone(),
                                  version: version_data.version_value.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        results
    }
}



pub fn pre_process_nvd_database<'a>(nvd: &'a NVD) -> HashMap<String, HashMap<String, Vec<SimpleCVEItem>>> {
    let mut ret: HashMap<String, HashMap<String, Vec<SimpleCVEItem>>> = HashMap::new();
    for cve_item in &nvd.CVE_Items {
        for vendor_data in &cve_item.cve.affects.vendor.vendor_data {
            for product_data in &vendor_data.product.product_data {
                let pkgname = &product_data.product_name;
                let node = ret.entry(pkgname.clone()).or_insert_with(|| HashMap::new());
                for version_data in &product_data.version.version_data {
                    let version_node = node.entry(version_data.version_value.clone())
                        .or_insert_with(|| Vec::new());
                    version_node.push(SimpleCVEItem{
                      pkg_name: pkgname.clone(),
                      cve_id: cve_item.cve.CVE_data_meta.ID.clone(),
                      version: version_data.version_value.clone(),
                    });
                }
            }
        }
    }
    ret
}

impl NVDSearchable for Vec<NVD> {
    fn get(&self, pkgname: &str, version: &str) -> Vec<SimpleCVEItem> {
        self.iter().map(|nvd| nvd.get(pkgname, version))
            .fold(vec![], |mut acc, v|{
                acc.extend(v);
                acc
            })
    }
}

pub fn get_nvd_databases(since: i32, update: bool) -> Result<Vec<NVD>, String> {
    let localTime: DateTime<Utc> = Utc::now();
    let mut nvds : Vec<NVD> = Vec::new();
    for year in since..localTime.date().year() + 1 {
        match get_nvd_database(year, update) {
            Ok(nvd) => nvds.push(nvd),
            Err(e) => return Err(e)
        }
    }

    Ok(nvds)
}

pub fn get_nvd_database(year: i32, update: bool) -> Result<NVD, String> {
    let url = format!(
        "https://static.nvd.nist.gov/feeds/json/cve/1.0/nvdcve-1.0-{}.json.gz",
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
