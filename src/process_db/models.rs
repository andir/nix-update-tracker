use super::schema::*;

#[derive(Debug, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "channels"]
pub struct Channel {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Serialize, Identifiable, Associations, PartialEq, Clone)]
#[table_name = "reports"]
#[belongs_to(Channel, foreign_key = "channel_id")]
pub struct Report {
    pub id: i32,
    pub date_time: String,
    pub commit_time: Option<String>,
    pub advance_time: Option<String>,
    pub revision: String,
    pub channel_id: i32,
}

#[derive(Insertable)]
#[table_name = "reports"]
pub struct NewReport {
    pub date_time: String,
    pub commit_time: Option<String>,
    pub advance_time: Option<String>,
    pub revision: String,
    pub channel_id: i32,
}


#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "packages"]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub attribute_name: String,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "package_versions"]
#[belongs_to(Package)]
pub struct PackageVersion {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "package_version_reports"]
#[belongs_to(PackageVersion)]
#[belongs_to(Report, foreign_key = "report_id")]
pub struct PackageVersionReport {
    pub id: i32,
    pub package_version_id: i32,
    pub report_id: i32,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "issues"]
pub struct Issue {
    pub id: i32,
    pub identifier: String,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "package_version_report_issues"]
#[belongs_to(Issue, foreign_key = "issue_id")]
#[belongs_to(PackageVersionReport, foreign_key = "package_version_report_id")]
pub struct PackageVersionReportIssue {
    pub id: i32,
    pub issue_id: i32,
    pub package_version_report_id: i32,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "patches"]
pub struct Patch {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "patches"]
pub struct NewPatch {
    pub name: String,
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "package_version_report_patches"]
#[belongs_to(PackageVersionReport, foreign_key = "package_version_report_id")]
#[belongs_to(Patch, foreign_key = "patch_id")]
pub struct PackageVersionReportPatch {
    pub id: i32,
    pub patch_id: i32,
    pub package_version_report_id: i32,
}
