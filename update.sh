#!/bin/sh

function only_cve() {
  jq "[to_entries[] | select(.value.cves != [])]|from_entries"
}
REV="master"
nix-update-tracker -u -s https://github.com/nixos/nixpkgs/archive/master.tar.gz -o unstable.json
only_cve < unstable.json > unstable-cve.json

REV="release-17.09"
URL="https://github.com/nixos/nixpkgs/archive/release-17.09.tar.gz"
nix-update-tracker -u -s ${URL} -o stable.json
only_cve < stable.json > stable-cve.json
