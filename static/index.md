---
pagetitle: Nix Vulnerability Scanner
---
# Nix Vulnerability Scanner

# Endpoints

- [`/channels`](channels) - a list of all known channels
- [`/channels/{name}`](channels/nixos-unstable) - information on a specific channel
- [`/channels/{name}/reports`](channels/nixos-unstable/reports) - a list of reports in the given channel
- [`/channels/{name}/reports/latest`](channels/nixos-unstable/reports/latest) - retrieve the latest report for the channel (newest advance date)
- [`/reports/{id}`](reports/1) - information on a specific report
- [`/issues`](issues) - list known issues
- [`/issues/{identifier}`](issues/CVE-2014-3627) - show one specific issue with affected channels & reports

The report contents are served by the following endpoints. They contain the same data. The difference is the grouping of information.

- [`/reports/{id}/issues`](reports/1/issues) - issues in a specific report
- [`/reports/{id}/packages`](reports/1/packages) - packages with issues in a specific report
