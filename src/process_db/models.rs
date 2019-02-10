use super::schema::*;
use diesel::sqlite::SqliteConnection;
//use diesel::RunQueryDsl;

use diesel::prelude::*;

#[derive(Debug, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "channels"]
pub struct Channel {
    pub id: i32,
    pub name: String,
}

impl Channel {
    pub fn create<T: AsRef<str>>(name: T, conn: &SqliteConnection) -> diesel::QueryResult<Channel> {
        #[derive(Insertable)]
        #[table_name = "channels"]
        struct NewChannel {
            name: String,
        }

        diesel::insert_into(channels::table)
            .values(&NewChannel {
                name: name.as_ref().to_string(),
            })
            .execute(conn)?;

        channels::table.order(channels::dsl::id.desc()).first(conn)
    }

    pub fn get<T: AsRef<str>>(name: T, conn: &SqliteConnection) -> diesel::QueryResult<Channel> {
        channels::dsl::channels
            .filter(channels::dsl::name.eq(name.as_ref()))
            .first::<Channel>(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        name: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Channel> {
        match Channel::get(&name, conn) {
            Ok(id) => Ok(id),
            Err(diesel::result::Error::NotFound) => Channel::create(&name, conn),
            Err(e) => Err(e),
        }
    }

    pub fn all(con: &SqliteConnection) -> diesel::QueryResult<Vec<Channel>> {
        channels::dsl::channels.load(con)
    }
}

#[derive(Debug, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "channel_bumps"]
#[belongs_to(Channel, foreign_key = "channel_id")]
#[belongs_to(Commit, foreign_key = "commit_id")]
pub struct ChannelBump {
    pub id: i32,
    pub channel_id: i32,
    pub commit_id: i32,
    pub channel_bump_date: i32,
}

impl ChannelBump {
    pub fn create(
        channel: &Channel,
        commit: &Commit,
        channel_bump_date: i32,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<ChannelBump> {
        #[derive(Insertable)]
        #[table_name = "channel_bumps"]
        struct NewChannelBump {
            channel_id: i32,
            commit_id: i32,
            channel_bump_date: i32,
        }

        diesel::insert_into(channel_bumps::table)
            .values(&NewChannelBump {
                channel_id: channel.id,
                commit_id: commit.id,
                channel_bump_date,
            })
            .execute(conn)?;
        channel_bumps::table
            .order(channel_bumps::dsl::id.desc())
            .first(conn)
    }
}

#[derive(Debug, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "commits"]
pub struct Commit {
    pub id: i32,
    pub revision: String,
    pub commit_time: Option<String>,
}

impl Commit {
    pub fn create<T: AsRef<str>>(
        revision: T,
        commit_time: Option<String>,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Commit> {
        #[derive(Insertable)]
        #[table_name = "commits"]
        struct Commit {
            revision: String,
            commit_time: Option<String>,
        }
        diesel::insert_into(commits::table)
            .values(&Commit {
                revision: revision.as_ref().to_string(),
                commit_time,
            })
            .execute(conn)?;
        commits::table.order(commits::dsl::id.desc()).first(conn)
    }

    pub fn get<T: AsRef<str>>(revision: T, conn: &SqliteConnection) -> diesel::QueryResult<Commit> {
        commits::dsl::commits
            .filter(commits::dsl::revision.eq(revision.as_ref()))
            .first(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        revision: T,
        commit_date: Option<String>,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Commit> {
        match Commit::get(&revision, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => Commit::create(&revision, commit_date, &conn),
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "issues"]
pub struct Issue {
    pub id: i32,
    pub identifier: String,
}

impl Issue {
    pub fn get<T: AsRef<str>>(
        identifier: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue> {
        issues::dsl::issues
            .filter(issues::dsl::identifier.eq(identifier.as_ref()))
            .first::<Issue>(conn)
    }

    pub fn create<T: AsRef<str>>(
        identifier: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue> {
        #[derive(Insertable)]
        #[table_name = "issues"]
        struct NewIssue {
            identifier: String,
        }

        diesel::insert_into(issues::table)
            .values(&NewIssue {
                identifier: identifier.as_ref().to_string(),
            })
            .execute(conn)?;
        issues::table.order(issues::dsl::id.desc()).first(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        identifier: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue> {
        match Issue::get(&identifier, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => Issue::create(&identifier, conn),
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "packages"]
pub struct Package {
    pub id: i32,
    pub name: String,
    pub attribute_name: String,
}

impl Package {
    pub fn get<T: AsRef<str>>(
        name: T,
        attribute_name: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Package> {
        packages::dsl::packages
            .filter(packages::dsl::name.eq(name.as_ref()))
            .filter(packages::dsl::attribute_name.eq(attribute_name.as_ref()))
            .first::<Package>(conn)
    }

    pub fn create<T: AsRef<str>>(
        name: T,
        attribute_name: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Package> {
        #[derive(Insertable)]
        #[table_name = "packages"]
        struct NewPackage {
            name: String,
            attribute_name: String,
        }

        diesel::insert_into(packages::table)
            .values(&NewPackage {
                name: name.as_ref().to_string(),
                attribute_name: attribute_name.as_ref().to_string(),
            })
            .execute(conn)?;
        packages::table.order(packages::dsl::id.desc()).first(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        name: T,
        attribute_name: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Package> {
        match Package::get(&name, &attribute_name, &conn) {
            Ok(p) => Ok(p),
            Err(diesel::result::Error::NotFound) => Package::create(&name, &attribute_name, &conn),
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone)]
#[table_name = "package_versions"]
#[belongs_to(Package)]
pub struct PackageVersion {
    pub id: i32,
    pub package_id: i32,
    pub version: String,
}

impl PackageVersion {
    pub fn get_for_pkg<T: AsRef<str>>(
        package: &Package,
        version: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion> {
        PackageVersion::belonging_to(package)
            .filter(package_versions::dsl::version.eq(version.as_ref()))
            .first::<PackageVersion>(conn)
    }

    pub fn create<T: AsRef<str>>(
        package: &Package,
        version: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion> {
        #[derive(Insertable)]
        #[table_name = "package_versions"]
        struct NewPackageVersion {
            package_id: i32,
            version: String,
        }

        diesel::insert_into(package_versions::table)
            .values(&NewPackageVersion {
                package_id: package.id,
                version: version.as_ref().to_string(),
            })
            .execute(conn)?;

        package_versions::table
            .order(package_versions::dsl::id.desc())
            .first(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        package: &Package,
        version: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion> {
        match PackageVersion::get_for_pkg(&package, &version, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => {
                PackageVersion::create(&package, &version, &conn)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "package_versions_in_commits"]
#[belongs_to(PackageVersion)]
#[belongs_to(Commit, foreign_key = "commit_id")]
pub struct PackageVersion2Commit {
    pub id: i32,
    pub package_version_id: i32,
    pub commit_id: i32,
}

impl PackageVersion2Commit {
    pub fn create(
        package_version: &PackageVersion,
        commit: &Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion2Commit> {
        #[derive(Insertable)]
        #[table_name = "package_versions_in_commits"]
        struct NewPackageVersion2Commit {
            package_version_id: i32,
            commit_id: i32,
        }

        diesel::insert_into(package_versions_in_commits::table)
            .values(&NewPackageVersion2Commit {
                package_version_id: package_version.id,
                commit_id: commit.id,
            })
            .execute(conn)?;
        package_versions_in_commits::table
            .order(package_versions_in_commits::dsl::id.desc())
            .first(conn)
    }

    pub fn get(
        package_version: &PackageVersion,
        commit: &Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion2Commit> {
        PackageVersion2Commit::belonging_to(package_version)
            .filter(package_versions_in_commits::dsl::commit_id.eq(commit.id))
            .first(conn)
    }

    pub fn get_or_create(
        package_version: &PackageVersion,
        commit: &Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<PackageVersion2Commit> {
        match PackageVersion2Commit::get(&package_version, &commit, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => {
                PackageVersion2Commit::create(package_version, commit, &conn)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "issues_in_versions"]
#[belongs_to(Issue, foreign_key = "issue_id")]
#[belongs_to(PackageVersion, foreign_key = "package_version_id")]
pub struct Issue2Version {
    pub id: i32,
    pub issue_id: i32,
    pub package_version_id: i32,
}

impl Issue2Version {
    pub fn get(
        issue: &Issue,
        package_version: &PackageVersion,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue2Version> {
        Issue2Version::belonging_to(issue)
            .filter(issues_in_versions::dsl::package_version_id.eq(package_version.id))
            .first(conn)
    }

    pub fn create(
        issue: &Issue,
        package_version: &PackageVersion,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue2Version> {
        #[derive(Insertable)]
        #[table_name = "issues_in_versions"]
        pub struct NewIssue2Version {
            issue_id: i32,
            package_version_id: i32,
        }

        diesel::insert_into(issues_in_versions::table)
            .values(&NewIssue2Version {
                issue_id: issue.id,
                package_version_id: package_version.id,
            })
            .execute(conn)?;
        issues_in_versions::table
            .order(issues_in_versions::dsl::id.desc())
            .first(conn)
    }

    pub fn get_or_create(
        issue: &Issue,
        package_version: &PackageVersion,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Issue2Version> {
        match Issue2Version::get(&issue, &package_version, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => {
                Issue2Version::create(&issue, &package_version, &conn)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, PartialEq, Clone, Debug)]
#[table_name = "patches"]
pub struct Patch {
    pub id: i32,
    pub name: String,
}

impl Patch {
    pub fn get<T: AsRef<str>>(name: T, conn: &SqliteConnection) -> diesel::QueryResult<Patch> {
        patches::dsl::patches
            .filter(patches::dsl::name.eq(name.as_ref()))
            .first(conn)
    }

    pub fn create<T: AsRef<str>>(name: T, conn: &SqliteConnection) -> diesel::QueryResult<Patch> {
        #[derive(Insertable)]
        #[table_name = "patches"]
        struct NewPatch {
            name: String,
        }

        diesel::insert_into(patches::table)
            .values(&NewPatch {
                name: name.as_ref().to_string(),
            })
            .execute(conn)?;

        patches::table.order(patches::dsl::id.desc()).first(conn)
    }

    pub fn get_or_create<T: AsRef<str>>(
        name: T,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Patch> {
        match Patch::get(&name, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => Patch::create(&name, &conn),
            Err(e) => Err(e),
        }
    }
}

#[derive(Identifiable, Serialize, Queryable, Associations, PartialEq, Clone, Debug)]
#[table_name = "patches_in_commits"]
#[belongs_to(Patch, foreign_key = "patch_id")]
#[belongs_to(PackageVersion2Commit, foreign_key = "package_commit_id")]
pub struct Patch2Commit {
    pub id: i32,
    pub patch_id: i32,
    pub package_commit_id: i32,
}

impl Patch2Commit {
    pub fn get(
        patch: &Patch,
        package_version_in_commit: &PackageVersion2Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Patch2Commit> {
        Patch2Commit::belonging_to(patch)
            .filter(patches_in_commits::dsl::package_commit_id.eq(package_version_in_commit.id))
            .first(conn)
    }

    pub fn create(
        patch: &Patch,
        package_version_in_commit: &PackageVersion2Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Patch2Commit> {
        #[derive(Insertable)]
        #[table_name = "patches_in_commits"]
        struct NewPatches2Commit {
            patch_id: i32,
            package_commit_id: i32,
        }

        diesel::insert_into(patches_in_commits::table)
            .values(&NewPatches2Commit {
                patch_id: patch.id,
                package_commit_id: package_version_in_commit.id,
            })
            .execute(conn)?;

        patches_in_commits::table
            .order(patches_in_commits::dsl::id.desc())
            .first(conn)
    }

    pub fn get_or_create(
        patch: &Patch,
        package_version_in_commit: &PackageVersion2Commit,
        conn: &SqliteConnection,
    ) -> diesel::QueryResult<Patch2Commit> {
        match Patch2Commit::get(&patch, &package_version_in_commit, &conn) {
            Ok(o) => Ok(o),
            Err(diesel::result::Error::NotFound) => {
                Patch2Commit::create(&patch, &package_version_in_commit, &conn)
            }
            Err(e) => Err(e),
        }
    }
}
