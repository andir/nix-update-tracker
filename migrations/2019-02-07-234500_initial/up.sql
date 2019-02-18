CREATE TABLE channels (
	id INTEGER PRIMARY KEY NOT NULL,
	name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE channel_bumps (
  id INTEGER PRIMARY KEY NOT NULL,
  channel_id INTEGER NOT NULL,
  commit_id INTEGER NOT NULL UNIQUE,
  channel_bump_date VARCHAR NOT NULL,
  FOREIGN KEY (channel_id) REFERENCES channels(id),
  FOREIGN KEY (commit_id) REFERENCES commits(id)
);

CREATE TABLE commits (
  id INTEGER PRIMARY KEY NOT NULL,
  revision VARCHAR NOT NULL UNIQUE,
  commit_time VARCHAR
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
	FOREIGN KEY (package_id) REFERENCES packages(id)
);

CREATE TABLE package_versions_in_commits (
  id INTEGER PRIMARY KEY NOT NULL,
  package_version_id INTEGER NOT NULL,
  commit_id INTEGER NOT NULL,
  FOREIGN KEY (package_version_id) REFERENCES package_versions(id),
  FOREIGN KEY (commit_id) REFERENCES commits(id)
);

CREATE TABLE issues (
  id INTEGER PRIMARY KEY NOT NULL,
  identifier VARCHAR NOT NULL UNIQUE
);

CREATE TABLE issues_in_versions (
  id INTEGER PRIMARY KEY NOT NULL,
  issue_id INTEGER NOT NULL,
  package_version_id INTEGER NOT NULL,
  FOREIGN KEY (issue_id) REFERENCES issues(id),
  FOREIGN KEY (package_Version_id) REFERENCES package_versions(id)
);

CREATE TABLE patches (
  id INTEGER PRIMARY KEY NOT NULL,
  name VARCHAR UNIQUE NOT NULL
);

CREATE TABLE patches_in_commits (
  id INTEGER PRIMARY KEY NOT NULL,
  package_commit_id INTEGER NOT NULL,
  patch_id INTEGER NOT NULL,
  FOREIGN KEY (package_commit_id) REFERENCES package_versions_in_commits(id),
  FOREIGN KEY (patch_id) REFERENCES patches(id)
);