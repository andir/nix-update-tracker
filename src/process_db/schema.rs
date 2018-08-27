table! {
    channels (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    issues (id) {
        id -> Integer,
        identifier -> Text,
    }
}

table! {
    package_version_report_issues (id) {
        id -> Integer,
        issue_id -> Integer,
        package_version_report_id -> Integer,
    }
}

table! {
    package_version_report_patches (id) {
        id -> Integer,
        patch_id -> Integer,
        package_version_report_id -> Integer,
    }
}

table! {
    package_version_reports (id) {
        id -> Integer,
        package_version_id -> Integer,
        report_id -> Integer,
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
    reports (id) {
        id -> Integer,
        date_time -> Text,
        commit_time -> Nullable<Text>,
        advance_time -> Nullable<Text>,
        revision -> Text,
        channel_id -> Integer,
    }
}

joinable!(package_version_report_issues -> issues (issue_id));
joinable!(package_version_report_issues -> package_version_reports (package_version_report_id));
joinable!(package_version_report_patches -> package_version_reports (package_version_report_id));
joinable!(package_version_report_patches -> patches (patch_id));
joinable!(package_version_reports -> package_versions (package_version_id));
joinable!(package_version_reports -> reports (report_id));
joinable!(package_versions -> packages (package_id));
joinable!(reports -> channels (channel_id));

allow_tables_to_appear_in_same_query!(
    channels,
    issues,
    package_version_report_issues,
    package_version_report_patches,
    package_version_reports,
    package_versions,
    packages,
    patches,
    reports,
);
