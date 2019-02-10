table! {
    channel_bumps (id) {
        id -> Integer,
        channel_id -> Integer,
        commit_id -> Integer,
        channel_bump_date -> Integer,
    }
}

table! {
    channels (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    commits (id) {
        id -> Integer,
        revision -> Text,
        commit_time -> Nullable<Text>,
    }
}

table! {
    issues (id) {
        id -> Integer,
        identifier -> Text,
    }
}

table! {
    issues_in_versions (id) {
        id -> Integer,
        issue_id -> Integer,
        package_version_id -> Integer,
    }
}

table! {
    package_versions (id) {
        id -> Integer,
        package_id -> Integer,
        version -> Text,
    }
}

table! {
    package_versions_in_commits (id) {
        id -> Integer,
        package_version_id -> Integer,
        commit_id -> Integer,
    }
}

table! {
    packages (id) {
        id -> Integer,
        name -> Text,
        attribute_name -> Text,
    }
}

table! {
    patches (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    patches_in_commits (id) {
        id -> Integer,
        package_commit_id -> Integer,
        patch_id -> Integer,
    }
}

joinable!(channel_bumps -> channels (channel_id));
joinable!(channel_bumps -> commits (commit_id));
joinable!(issues_in_versions -> issues (issue_id));
joinable!(issues_in_versions -> packages (package_version_id));
joinable!(package_versions -> packages (package_id));
joinable!(package_versions_in_commits -> commits (commit_id));
joinable!(package_versions_in_commits -> package_versions (package_version_id));
joinable!(patches_in_commits -> package_versions_in_commits (package_commit_id));
joinable!(patches_in_commits -> patches (patch_id));

allow_tables_to_appear_in_same_query!(
    channel_bumps,
    channels,
    commits,
    issues,
    issues_in_versions,
    package_versions,
    package_versions_in_commits,
    packages,
    patches,
    patches_in_commits,
);
