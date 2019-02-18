//use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde_json;
use std::fs;

use self::models::*;
use super::report::Report;

pub mod models;
pub mod schema;

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

    fn add_package(
        &self,
        commit: &models::Commit,
        package: &super::report::Package,
    ) -> Result<(models::PackageVersion, models::PackageVersion2Commit)> {
        let pkg = models::Package::get_or_create(
            &package.package_name,
            &package.attribute_name,
            &self.con,
        )
        .chain_err(|| "Failed to get or create package in database")?;

        let pkg_version = models::PackageVersion::get_or_create(&pkg, &package.version, &self.con)
            .chain_err(|| "Failed to get or create package version in database")?;

        let pkg_version_in_commit =
            models::PackageVersion2Commit::get_or_create(&pkg_version, &commit, &self.con)
                .chain_err(|| "Failed to get or create package version to commit link")?;

        Ok((pkg_version, pkg_version_in_commit))
    }

    fn add_patches(
        &self,
        package_version_to_commit: &models::PackageVersion2Commit,
        patches: &[String],
    ) -> Result<()> {
        for patch in patches.iter() {
            let patch = models::Patch::get_or_create(patch, &self.con)?;
            Patch2Commit::get_or_create(&patch, &package_version_to_commit, &self.con)?;
        }

        Ok(())
    }

    fn add_issues(
        &self,
        package_version: &models::PackageVersion,
        cves: &[super::report::CVE],
    ) -> Result<()> {
        for cve in cves.iter() {
            let issue = models::Issue::get_or_create(&cve.name, &self.con)
                .chain_err(|| "Failed to get or create issue")?;

            models::Issue2Version::get_or_create(&issue, &package_version, &self.con)
                .chain_err(|| "Failed to insert link between issue and package version")?;
        }

        Ok(())
    }

    pub fn add_report(&self, report: &Report) -> Result<()> {
        self.con.transaction::<_, Error, _>(|| {
            // channel is actually not required here, we do the channel and channel bump tracking elsewhere
            //let channel = models::Channel::get_or_create(&report.channel_name, &self.con)?;

            let commit = models::Commit::get_or_create(
                &report.revision,
                report.commit_time.clone(),
                &self.con,
            )?;

            println!("commit: {}", commit.revision);

            //let report = self.get_or_create_report(&report, channel.id)?;
            for package in report.packages.values() {
                let (package_version, pkg_version_in_commit) =
                    self.add_package(&commit, &package)?;
                self.add_issues(&package_version, &package.cves)?;
                self.add_patches(&pkg_version_in_commit, &package.patches)?;
            }

            Ok(())
        })?;

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
