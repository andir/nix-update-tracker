use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde_json;
use std::fs;

use self::models::*;
use super::report::Report;

mod models;
mod schema;
pub mod web;

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

        let query_pkg = |name| {
            let f = pkgs_dsl::name.eq(&name);
            pkgs_dsl::packages.filter(f).first::<Package>(&self.con)
        };

        let pkg = match query_pkg(&package.package_name) {
            Ok(p) => p,
            Err(diesel::NotFound) => {
                diesel::insert_into(pkgs_dsl::packages)
                    .values((
                        pkgs_dsl::name.eq(&package.package_name),
                        pkgs_dsl::attribute_name.eq(&package.attribute_name),
                    ))
                    .execute(&self.con)
                    .chain_err(|| "failed to insert new package")?;
                query_pkg(&package.package_name)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "Failed to lookup package");
            }
        };

        // check if the version already exists

        let query_version = |pkg: &Package, package: &super::report::Package| {
            PackageVersion::belonging_to(pkg)
                .select(version_dsl::id)
                .filter(version_dsl::version.eq(&package.version))
                .first::<i32>(&self.con)
        };

        let version_id = match query_version(&pkg, package) {
            Ok(version) => version,
            Err(diesel::NotFound) => {
                diesel::insert_into(version_dsl::package_versions)
                    .values((
                        version_dsl::package_id.eq(pkg.id),
                        version_dsl::version.eq(&package.version),
                    ))
                    .execute(&self.con)
                    .chain_err(|| "failed to insert version to database")?;
                query_version(&pkg, package)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "Failed to lookup package version");
            }
        };

        // link the version to the report
        let query_link = |report_id, version_id| {
            pkgversion_dsl::package_version_reports
                .select(pkgversion_dsl::id)
                .filter(pkgversion_dsl::report_id.eq(report_id))
                .filter(pkgversion_dsl::package_version_id.eq(version_id))
                .first::<i32>(&self.con)
        };

        let link_id = match query_link(report_id, version_id) {
            Ok(id) => id,
            Err(diesel::NotFound) => {
                let values = (
                    pkgversion_dsl::report_id.eq(report_id),
                    pkgversion_dsl::package_version_id.eq(version_id),
                );
                diesel::insert_into(pkgversion_dsl::package_version_reports)
                    .values(values)
                    .execute(&self.con)
                    .chain_err(|| "failed to add package_version_reports entry")?;
                query_link(report_id, version_id)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "Failed to lookup package version report");
            }
        };

        Ok(link_id)
    }

    fn get_or_create_channel(&self, channel: &str) -> Result<i32> {
        use self::schema::channels::dsl as channel_dsl;
        let query = || {
            let q = channel_dsl::channels
                .select(channel_dsl::id)
                .filter(channel_dsl::name.eq(&channel))
                .first::<i32>(&self.con);
            return q;
        };

        return match query() {
            Ok(id) => Ok(id),
            Err(diesel::result::Error::NotFound) => {
                diesel::insert_into(channel_dsl::channels)
                    .values((channel_dsl::name.eq(channel),))
                    .execute(&self.con)
                    .chain_err(|| "failed to insert channel")?;
                Ok(query()?)
            }
            Err(e) => Err(e).chain_err(|| format!("Failed to query for channel {}", channel)),
        };
    }

    fn get_or_create_patch(&self, patch: &String) -> Result<i32> {
        use self::models::NewPatch;
        use self::schema::patches::dsl as patch_dsl;

        let patch_query = |patch_name| {
            patch_dsl::patches
                .select(patch_dsl::id)
                .filter(patch_dsl::name.eq(patch_name))
                .first::<i32>(&self.con)
        };

        let patch_id = match patch_query(&patch) {
            Ok(id) => id,
            Err(diesel::NotFound) => {
                let p = NewPatch {
                    name: patch.clone(),
                };

                diesel::insert_into(patch_dsl::patches)
                    .values(&p)
                    .execute(&self.con)
                    .chain_err(|| "failed to insert new patch")?;
                patch_query(&patch)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "failed to lookup patch");
            }
        };

        Ok(patch_id)
    }

    fn get_or_create_report(&self, report: &Report, channel_id: i32) -> Result<i32> {
        use self::models::NewReport;
        use self::schema::reports::dsl as report_dsl;

        let report_query = |revision, channel_id| {
            report_dsl::reports
                .select(report_dsl::id)
                .filter(report_dsl::revision.eq(&revision))
                .filter(report_dsl::channel_id.eq(channel_id))
                .first::<i32>(&self.con)
        };

        let report_id = match report_query(&report.revision, channel_id) {
            Ok(id) => id,
            Err(diesel::NotFound) => {
                let r = NewReport {
                    date_time: report.date_time.clone(),
                    commit_time: report.commit_time.clone(),
                    advance_time: report.advance_time.clone(),
                    revision: report.revision.clone(),
                    channel_id: channel_id,
                };
                diesel::insert_into(report_dsl::reports)
                    .values(&r)
                    .execute(&self.con)
                    .chain_err(|| "failed to insert report")?;
                report_query(&report.revision, channel_id)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "failed to lookup report");
            }
        };

        Ok(report_id)
    }

    fn link_package_version_report_to_patch(
        &self,
        package_version_report_id: i32,
        patch_id: i32,
    ) -> Result<i32> {
        use self::schema::package_version_report_patches::dsl as pkgvrp_patches_dsl;
        let link_query = |pkvr_id, patch_id| {
            pkgvrp_patches_dsl::package_version_report_patches
                .select(pkgvrp_patches_dsl::id)
                .filter(pkgvrp_patches_dsl::patch_id.eq(patch_id))
                .filter(pkgvrp_patches_dsl::package_version_report_id.eq(package_version_report_id))
                .first::<i32>(&self.con)
        };

        let id = match link_query(package_version_report_id, patch_id) {
            Ok(id) => id,
            Err(diesel::NotFound) => {
                diesel::insert_into(pkgvrp_patches_dsl::package_version_report_patches)
                    .values((
                        pkgvrp_patches_dsl::patch_id.eq(&patch_id),
                        pkgvrp_patches_dsl::package_version_report_id
                            .eq(&package_version_report_id),
                    ))
                    .execute(&self.con)
                    .chain_err(|| "failed to insert new package_version_report_patches entry")?;
                link_query(package_version_report_id, patch_id)?
            }
            Err(e) => {
                return Err(e).chain_err(|| "failed to query for package_version_report_patches");
            }
        };

        Ok(id)
    }

    fn add_patches(&self, package_version_report_id: i32, patches: &Vec<String>) -> Result<()> {
        for patch in patches.iter() {
            let patch_id = self.get_or_create_patch(patch)?;
            self.link_package_version_report_to_patch(package_version_report_id, patch_id)?;
        }

        Ok(())
    }

    fn add_issues(
        &self,
        package_version_report_id: i32,
        cves: &Vec<super::report::CVE>,
    ) -> Result<()> {
        use self::schema::issues::dsl as issue_dsl;
        use self::schema::package_version_report_issues::dsl as pkg_ver_report_issues_dsl;

        let issue_query = |name| {
            let issue = issue_dsl::issues
                .select(issue_dsl::id)
                .filter(issue_dsl::identifier.eq(&name))
                .first::<i32>(&self.con);
            issue
        };

        for cve in cves.iter() {
            let issue_id = match issue_query(&cve.name) {
                Err(diesel::NotFound) => {
                    diesel::insert_into(issue_dsl::issues)
                        .values(issue_dsl::identifier.eq(&cve.name))
                        .execute(&self.con)
                        .chain_err(|| "Failed to add issue")?;
                    issue_query(&cve.name)?
                }
                Ok(id) => id,
                Err(e) => {
                    return Err(e).chain_err(|| "Failed to search for issue");
                }
            };

            // check for the link between the issue and the current package_version_report_id

            let links_query = pkg_ver_report_issues_dsl::package_version_report_issues
                .select(pkg_ver_report_issues_dsl::id)
                .filter(
                    pkg_ver_report_issues_dsl::package_version_report_id
                        .eq(package_version_report_id),
                )
                .filter(pkg_ver_report_issues_dsl::issue_id.eq(issue_id))
                .count();

            let links: i64 = links_query
                .get_result(&self.con)
                .chain_err(|| "failed to retrieve links")?;

            if links == 0 {
                let insert_stmt = diesel::insert_into(
                    pkg_ver_report_issues_dsl::package_version_report_issues,
                ).values((
                    pkg_ver_report_issues_dsl::package_version_report_id
                        .eq(package_version_report_id),
                    pkg_ver_report_issues_dsl::issue_id.eq(issue_id),
                ));

                insert_stmt
                    .execute(&self.con)
                    .chain_err(|| "failed to add package version report issues entry")?;
            };
        }

        Ok(())
    }

    pub fn add_report(&self, report: &Report) -> Result<()> {
        let channel_id = self.get_or_create_channel(&report.channel_name)?;
        let report_id = self.get_or_create_report(&report, channel_id)?;
        for (_, package) in &report.packages {
            let package_version_report_id = self.add_package(report_id, &package)?;
            self.add_issues(package_version_report_id, &package.cves)?;
            self.add_patches(package_version_report_id, &package.patches)?;
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
