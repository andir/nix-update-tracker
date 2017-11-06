extern crate libflate;
extern crate serde_json;

use fs_cache;
use reqwest;
use std;
use std::io::Read;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Affects {
    vendor: Vendor,
}

#[derive(Serialize, Deserialize)]
pub struct BaseMetricV2 {
    cvssV2: CvssV2,
    severity: String,
    exploitabilityScore: f64,
    impactScore: f64,
    obtainAllPrivilege: bool,
    obtainUserPrivilege: bool,
    obtainOtherPrivilege: bool,
    userInteractionRequired: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BaseMetricV3 {
    cvssV3: CvssV3,
    exploitabilityScore: f64,
    impactScore: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Configurations {
    CVE_data_version: String,
    nodes: Vec<Nodes>,
}

#[derive(Serialize, Deserialize)]
pub struct Cpe {
    vulnerable: bool,
    previousVersions: Option<bool>,
    cpeMatchString: String,
    cpe23Uri: String,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct CveDataMeta {
    ID: String,
}

#[derive(Serialize, Deserialize)]
pub struct CveItems {
    cve: Cve,
    configurations: Configurations,
    impact: Impact,
    publishedDate: String,
    lastModifiedDate: String,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Description {
    lang: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Description1 {
    description_data: Vec<Description>,
}

#[derive(Serialize, Deserialize)]
pub struct Impact {
    baseMetricV3: Option<BaseMetricV3>,
    baseMetricV2: Option<BaseMetricV2>,
}

#[derive(Serialize, Deserialize)]
pub struct Nodes {
    operator: String,
    negate: Option<bool>,
    children: Option<Vec<Nodes>>,
    cpe: Option<Vec<Cpe>>,
}

#[derive(Serialize, Deserialize)]
pub struct Problemtype {
    problemtype_data: Vec<ProblemtypeData>,
}

#[derive(Serialize, Deserialize)]
pub struct ProblemtypeData {
    description: Vec<Description>,
}

#[derive(Serialize, Deserialize)]
pub struct Product {
    product_data: Vec<ProductData>,
}

#[derive(Serialize, Deserialize)]
pub struct ProductData {
    product_name: String,
    version: Version,
}

#[derive(Serialize, Deserialize)]
pub struct ReferenceData {
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct References {
    reference_data: Vec<ReferenceData>,
}

#[derive(Serialize, Deserialize)]
pub struct NVD {
    CVE_data_type: String,
    CVE_data_format: String,
    CVE_data_version: String,
    CVE_data_numberOfCVEs: String,
    CVE_data_timestamp: String,
    CVE_Items: Vec<CveItems>,
}

#[derive(Serialize, Deserialize)]
pub struct Vendor {
    vendor_data: Vec<VendorData>,
}

#[derive(Serialize, Deserialize)]
pub struct VendorData {
    vendor_name: String,
    product: Product,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    version_data: Vec<VersionData>,
}

#[derive(Serialize, Deserialize)]
pub struct VersionData {
    version_value: String,
}

impl NVD {
    pub fn get(&self, pkgname: &str, version: &str) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        for cveItem in &self.CVE_Items {
            for vendor_data in &cveItem.cve.affects.vendor.vendor_data {
                for product_data in &vendor_data.product.product_data {
                    if product_data.product_name == pkgname {
                        for version_data in &product_data.version.version_data {
                            if version_data.version_value == version {
                                results.push(cveItem.cve.CVE_data_meta.ID.clone());
                            }
                        }
                    }
                }
            }
        }
        results
    }
}


pub fn pre_process_nvd_database(nvd: &NVD) -> HashMap<String, HashMap<String, Vec<String>>> {
    let mut ret: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    for cveItem in &nvd.CVE_Items {
        for vendor_data in &cveItem.cve.affects.vendor.vendor_data {
            for product_data in &vendor_data.product.product_data {
                let pkgname = &product_data.product_name;
                let node = ret.entry(pkgname.clone()).or_insert_with(|| HashMap::new());
                for version_data in &product_data.version.version_data {
                    let version_node = node.entry(version_data.version_value.clone())
                        .or_insert_with(|| Vec::new());
                    let cveID = cveItem.cve.CVE_data_meta.ID.clone();
                    if !version_node.contains(&cveID) {
                        version_node.push(cveID);
                    }
                }
            }
        }
    }
    ret
}

pub fn get_nvd_database(year: i32, update: bool) -> Result<NVD, String> {
    let url = format!(
        "https://static.nvd.nist.gov/feeds/json/cve/1.0/nvdcve-1.0-{}.json.gz",
        year
    );
    let mut body: Vec<u8> = Vec::new();

    let data = match fs_cache::http_get_cached(&format!("nvd-{}.json.gz", year), &url, update) {
        Err(e) => return Err(e),
        Ok(o) => o,
    };

    let mut decompressed_body = Vec::new();
    let mut decoder = libflate::gzip::Decoder::new(&data[..]).unwrap();
    let mut decoded_data: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut decompressed_body);

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
