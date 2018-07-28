use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde_json;
use std::fs;

use self::models::*;
use super::report::Report;

mod models;
mod schema;

error_chain! {
    foreign_links {
        Serde(::serde_json::Error);
        DieselConnection(::diesel::ConnectionError);
        DieselResultError(::diesel::result::Error);
    }
}

fn database_connection(dbfile: &str) -> Result<SqliteConnection> {
    Ok(SqliteConnection::establish(&dbfile)?)
}

struct SqliteDatabase {
    con: SqliteConnection,
}

impl SqliteDatabase {
    pub fn new(dbfile: &str) -> Result<Self> {
        Ok(Self {
            con: database_connection(dbfile)?,
        })
    }

    fn add_package(&self, report_id: i32, package: &super::report::Package) -> Result<i32> {
        // check if the package is already known
        use self::schema::package_version_reports::dsl as pkgversion_dsl;
        use self::schema::package_versions::dsl as version_dsl;
        use self::schema::packages::dsl as pkgs_dsl;

        let f = pkgs_dsl::name.eq(&package.package_name);
        let pkgs = pkgs_dsl::packages
            .filter(f)
            .limit(1)
            .load::<Package>(&self.con)?;
        let pkgs = if pkgs.len() == 0 {
            // insert
            diesel::insert_into(pkgs_dsl::packages)
                .values((
                    pkgs_dsl::name.eq(&package.package_name),
                    pkgs_dsl::attribute_name.eq(&package.attribute_name),
                ))
                .execute(&self.con)
                .chain_err(|| "failed to insert new package")?;
            pkgs_dsl::packages
                .filter(f)
                .limit(1)
                .load::<Package>(&self.con)?
        } else {
            pkgs
        };
        let pkg = pkgs.iter()
            .nth(0)
            .chain_err(|| "expected the first element to exist")?;

        // check if the version already exists
        let versions = PackageVersion::belonging_to(&pkgs)
            .filter(version_dsl::version.eq(&package.version))
            .limit(1)
            .load::<PackageVersion>(&self.con)?;
        let versions = if versions.len() == 0 {
            diesel::insert_into(version_dsl::package_versions)
                .values((
                    version_dsl::package_id.eq(pkg.id),
                    version_dsl::version.eq(&package.version),
                ))
                .execute(&self.con)
                .chain_err(|| "failed to insert version to database")?;
            PackageVersion::belonging_to(&pkgs)
                .filter(version_dsl::version.eq(&package.version))
                .limit(1)
                .load::<PackageVersion>(&self.con)?
        } else {
            versions
        };

        let version = versions
            .iter()
            .nth(0)
            .chain_err(|| "failed to retreive package version entry")?;

        // link the version to the report
        let report_id_f = pkgversion_dsl::report_id.eq(report_id);
        let package_version_id_f = pkgversion_dsl::package_version_id.eq(version.id);

        let links = pkgversion_dsl::package_version_reports
            .filter(report_id_f)
            .filter(package_version_id_f)
            .limit(1)
            .load::<PackageVersionReport>(&self.con)?;

        let links = if links.len() == 0 {
            let values = (&report_id_f, &package_version_id_f);
            diesel::insert_into(pkgversion_dsl::package_version_reports)
                .values(values)
                .execute(&self.con)
                .chain_err(|| "failed to add package_version_reports entry")?;
            pkgversion_dsl::package_version_reports
                .filter(report_id_f)
                .filter(package_version_id_f)
                .limit(1)
                .load::<PackageVersionReport>(&self.con)?
        } else {
            links
        };

        let link = links
            .iter()
            .nth(0)
            .chain_err(|| "failed to get package_version_reports entry")?;

        Ok(link.id)
    }

    fn get_or_create_report(&self, report: &Report) -> Result<i32> {
        use self::schema::reports::dsl as report_dsl;

        let reports = report_dsl::reports
            .filter(report_dsl::revision.eq(&report.revision))
            .limit(1)
            .load::<self::models::Report>(&self.con)?;
        let reports = if reports.len() == 0 {
            diesel::insert_into(report_dsl::reports)
                .values((
                    report_dsl::date_time.eq(&report.date_time),
                    report_dsl::repository.eq("fixme"),
                    report_dsl::revision.eq(&report.revision),
                ))
                .execute(&self.con)
                .chain_err(|| "failed to insert report")?;
            report_dsl::reports
                .filter(report_dsl::revision.eq(&report.revision))
                .limit(1)
                .load::<self::models::Report>(&self.con)?
        } else {
            reports
        };
        let report = reports.iter().nth(0).chain_err(|| "Report not found")?;

        Ok(report.id)
    }

    fn add_issues(
        &self,
        package_version_report_id: i32,
        cves: &Vec<super::report::CVE>,
    ) -> Result<()> {
        use self::schema::issues::dsl as issue_dsl;
        use self::schema::package_version_report_issues::dsl as pkg_ver_report_issues_dsl;
        for cve in cves.iter() {
            let issues = issue_dsl::issues
                .filter(issue_dsl::identifier.eq(&cve.name))
                .limit(1)
                .load::<self::models::Issue>(&self.con)?;

            let issues = if issues.len() == 0 {
                diesel::insert_into(issue_dsl::issues)
                    .values(issue_dsl::identifier.eq(&cve.name))
                    .execute(&self.con)
                    .chain_err(|| "Failed to add issue")?;
                issue_dsl::issues
                    .filter(issue_dsl::identifier.eq(&cve.name))
                    .limit(1)
                    .load::<self::models::Issue>(&self.con)?
            } else {
                issues
            };

            let issue = issues
                .iter()
                .nth(0)
                .chain_err(|| "failed to get issue entry")?;

            // check for the link between the issue and the current package_version_report_id

            let links = pkg_ver_report_issues_dsl::package_version_report_issues
                .filter(
                    pkg_ver_report_issues_dsl::package_version_report_id
                        .eq(package_version_report_id),
                )
                .filter(pkg_ver_report_issues_dsl::issue_id.eq(issue.id))
                .count()
                .execute(&self.con)
                .chain_err(|| "failed to retrieve links")?;

            if links == 0 {
                diesel::insert_into(pkg_ver_report_issues_dsl::package_version_report_issues)
                    .values((
                        pkg_ver_report_issues_dsl::package_version_report_id
                            .eq(package_version_report_id),
                        pkg_ver_report_issues_dsl::issue_id.eq(issue.id),
                    ))
                    .execute(&self.con)
                    .chain_err(|| "failed to add package version report issues entry")?;
            };
        }

        Ok(())
    }

    pub fn add_report(&self, report: &Report) -> Result<()> {
        let report_id = self.get_or_create_report(&report)?;
        for (_, package) in &report.packages {
            let package_version_report_id = self.add_package(report_id, &package)?;
            self.add_issues(package_version_report_id, &package.cves);
        }
        Ok(())
    }
}

pub trait Database {
    fn add_report(&self, report: &Report) -> Result<()>;
}

pub fn import(dbfile: &str, source: &str) -> Result<()> {
    let db = SqliteDatabase::new(dbfile)?;

    // open the input file
    let fh: fs::File = fs::File::open(source).chain_err(|| "failed to open input file")?;

    let input: Report = serde_json::from_reader(fh)?;

    db.add_report(&input)?;

    Ok(())
}
