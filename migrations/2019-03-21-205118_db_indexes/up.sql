-- Your SQL goes here
CREATE INDEX package_versions_in_commits_commit_version ON package_versions_in_commits (commit_id, package_version_id);
CREATE INDEX patches_in_commits_patch_id_package_commit_id ON patches_in_commits (patch_id, package_commit_id);
CREATE INDEX issues_in_versions_id_pkg_ver_id ON issues_in_versions (issue_id, package_version_id);
CREATE INDEX package_versions_version ON package_versions (package_id, version);