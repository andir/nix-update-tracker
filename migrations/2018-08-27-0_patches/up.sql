CREATE TABLE patches (
	id INTEGER PRIMARY KEY NOT NULL,
	name VARCHAR NOT NULL UNIQUE
);

CREATE TABLE package_version_report_patches (
	id INTEGER PRIMARY KEY NOT NULL,
	patch_id INTEGER NOT NULL,
	package_version_report_id INTEGER NOT NULL,

	FOREIGN KEY(patch_id) REFERENCES patches(id),
	FOREIGN KEY(package_version_report_id) REFERENCES package_version_reports(id)
);
