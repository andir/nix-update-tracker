use super::schema::*;

#[derive(Queryable, Associations)]
#[table_name = "channels"]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Insertable)]
#[table_name = "channels"]
pub struct NewChannel {
    pub name: String,
    pub url: String,
}

#[derive(Queryable, Associations)]
#[table_name = "reports"]
pub struct Report {
    pub id: i32,
    pub date_time: String,
    pub revision: String,
    pub repository: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq)]
#[table_name = "packages"]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub attribute_name: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "package_versions"]
#[belongs_to(Package)]
pub struct PackageVersion {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "package_version_reports"]
#[belongs_to(PackageVersion)]
#[belongs_to(Report, foreign_key = "report_id")]
pub struct PackageVersionReport {
    pub id: i32,
    pub package_version_id: i32,
    pub report_id: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "issues"]
pub struct Issue {
    pub id: i32,
    pub identifier: String,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "package_version_report_issues"]
#[belongs_to(Issue, foreign_key = "issue_id")]
#[belongs_to(PackageVersionReport, foreign_key = "package_version_report_id")]
pub struct PackageVersionReportIssue {
    pub id: i32,
    pub issue_id: i32,
    pub package_version_report_id: i32,
}
