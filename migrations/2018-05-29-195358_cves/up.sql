CREATE TABLE channels (
	id INTEGER PRIMARY KEY NOT NULL,
	name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE reports (
	id INTEGER PRIMARY KEY NOT NULL,
	date_time VARCHAR NOT NULL,
	commit_time VARCHAR,
	advance_time VARCHAR,
	revision VARCHAR NOT NULL,
	channel_id INTEGER NOT NULL,
	FOREIGN KEY(channel_id) REFERENCES channels(id)
);

CREATE TABLE packages (
	id INTEGER PRIMARY KEY NOT NULL,
	name VARCHAR NOT NULL,
	attribute_name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE package_versions (
	id INTEGER PRIMARY KEY NOT NULL,
	package_id INTEGER NOT NULL,
	version VARCHAR NOT NULL,
	FOREIGN KEY(package_id) REFERENCES packages(id)
);

CREATE TABLE package_version_reports (
	id INTEGER PRIMARY KEY NOT NULL,
	package_version_id INTEGER NOT NULL,
	report_id INTEGER NOT NULL,
	FOREIGN KEY(package_version_id) REFERENCES package_versions(id),
	FOREIGN KEY(report_id) REFERENCES reports(id)
);

CREATE TABLE issues (
	id INTEGER PRIMARY KEY NOT NULL,
	identifier VARCHAR NOT NULL
);

CREATE TABLE package_version_report_issues (
	id INTEGER PRIMARY KEY NOT NULL,
	issue_id INTEGER NOT NULL,
	package_version_report_id INTEGER NOT NULL,

	FOREIGN KEY(issue_id) REFERENCES issues(id),
	FOREIGN KEY(package_version_report_id) REFERENCES package_version_reports(id)
)
