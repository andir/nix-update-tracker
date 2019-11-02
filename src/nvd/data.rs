use super::version_cmp::VersionCmp;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Affects {
    pub vendor: Vendor,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV2 {
    #[serde(rename = "cvssV2")]
    pub cvss_v2: CvssV2,
    pub severity: String,
    #[serde(rename = "exploitabilityScore")]
    pub exploitability_score: f64,
    #[serde(rename = "impactScore")]
    pub impact_score: f64,
    #[serde(rename = "obtainAllPrivilege")]
    pub obtain_all_privilege: bool,
    #[serde(rename = "obtainUserPrivilege")]
    pub obtain_user_privilege: bool,
    #[serde(rename = "obtainOtherPrivilege")]
    pub obtain_other_privilege: bool,
    #[serde(rename = "userInteractionRequired")]
    pub user_interaction_required: Option<bool>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BaseMetricV3 {
    #[serde(rename = "cvssV3")]
    pub cvss_v3: CvssV3,
    #[serde(rename = "exploitabilityScore")]
    pub exploitability_score: f64,
    #[serde(rename = "impactScore")]
    pub impact_score: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Configurations {
    #[serde(rename = "CVE_data_version")]
    pub cve_data_version: String,
    pub nodes: Vec<Nodes>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cpe {
    vulnerable: bool,
    #[serde(rename = "previousVersions")]
    pub previous_versions: Option<bool>,
    #[serde(rename = "cpeMatchString")]
    pub cpe_match_string: Option<String>,
    #[serde(rename = "cpe23Uri")]
    pub cpe23_uri: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Cve {
    pub data_type: String,
    pub data_format: String,
    pub data_version: String,
    #[serde(rename = "CVE_data_meta")]
    pub cve_data_meta: CveDataMeta,
    pub affects: Affects,
    pub problemtype: Problemtype,
    pub references: References,
    pub description: Description1,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveDataMeta {
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CveItems {
    pub cve: Cve,
    pub configurations: Configurations,
    pub impact: Impact,
    #[serde(rename = "publishedDate")]
    pub published_date: String,
    #[serde(rename = "lastModifiedDate")]
    pub last_modified_date: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV2 {
    #[serde(rename = "vectorString")]
    pub vector_string: String,
    #[serde(rename = "accessVector")]
    pub access_vector: String,
    #[serde(rename = "accessComplexity")]
    pub access_complexity: String,
    pub authentication: String,
    #[serde(rename = "confidentialityImpact")]
    pub confidentiality_impact: String,
    #[serde(rename = "integrityImpact")]
    pub integrity_impact: String,
    #[serde(rename = "availabilityImpact")]
    pub availability_impact: String,
    #[serde(rename = "baseScore")]
    pub base_score: f64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CvssV3 {
    #[serde(rename = "vectorString")]
    pub vector_string: String,
    #[serde(rename = "attackVector")]
    pub attack_vector: String,
    #[serde(rename = "attackComplexity")]
    pub attack_complexity: String,
    #[serde(rename = "privilegesRequired")]
    pub privileges_required: String,
    #[serde(rename = "userInteraction")]
    pub user_interaction: String,
    pub scope: String,
    #[serde(rename = "confidentialityImpact")]
    pub confidentiality_impact: String,
    #[serde(rename = "integrityImpact")]
    pub integrity_impact: String,
    #[serde(rename = "availabilityImpact")]
    pub availability_impact: String,
    #[serde(rename = "baseScore")]
    pub base_score: f64,
    #[serde(rename = "baseSeverity")]
    pub base_severity: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Description {
    pub lang: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Description1 {
    pub description_data: Vec<Description>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Impact {
    #[serde(rename = "baseMetricV3")]
    pub base_metric_v3: Option<BaseMetricV3>,
    #[serde(rename = "baseMetricV2")]
    pub base_metric_v2: Option<BaseMetricV2>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Nodes {
    pub operator: String,
    pub negate: Option<bool>,
    pub children: Option<Vec<Nodes>>,
    pub cpe: Option<Vec<Cpe>>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Problemtype {
    pub problemtype_data: Vec<ProblemtypeData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ProblemtypeData {
    pub description: Vec<Description>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Product {
    pub product_data: Vec<ProductData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ProductData {
    pub product_name: String,
    pub version: Version,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ReferenceData {
    pub url: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct References {
    pub reference_data: Vec<ReferenceData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NVD {
    #[serde(rename = "CVE_data_type")]
    pub cve_data_type: String,
    #[serde(rename = "CVE_data_format")]
    pub cve_data_format: String,
    #[serde(rename = "CVE_data_version")]
    pub cve_data_version: String,
    #[serde(rename = "CVE_data_numberOfCVEs")]
    pub cve_data_number_of_cves: String,
    #[serde(rename = "CVE_data_timestamp")]
    pub cve_data_timestamp: String,
    #[serde(rename = "CVE_Items")]
    pub cve_items: Vec<CveItems>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Vendor {
    pub vendor_data: Vec<VendorData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VendorData {
    pub vendor_name: String,
    pub product: Product,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Version {
    pub version_data: Vec<VersionData>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VersionData {
    pub version_value: String,
    pub version_affected: Option<String>,
}

impl VersionData {
    pub fn matches<T: AsRef<str>>(&self, other: T) -> bool {
        let other = other.as_ref();
        match self.version_affected.as_ref() {
            None => self.version_value == other.as_ref(),
            Some(x) if x == "<=" => other.version_le(&self.version_value),
            Some(x) if x == "=" => other.version_eq(&self.version_value),
            Some(other) => {
                error!("Unknown match type {}", other);
                return false;
            }
        }
    }
}

