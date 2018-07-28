table! {
    channels (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
    }
}

table! {
    reports (id) {
        id -> Integer,
        date_time -> Text,
        revision -> Text,
        repository -> Text,
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
    package_versions (id) {
        id -> Integer,
        package_id -> Integer,
        version -> Text,
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
    issues (id) {
        id -> Integer,
        identifier -> Text,
    }
}

table! {
    package_version_report_issues {
        id -> Integer,
        issue_id -> Integer,
        package_version_report_id -> Integer,
    }
}
