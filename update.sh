#!/bin/sh

set -ex

function only_cve() {
  jq "[to_entries[] | select(.value.cves != [])]|from_entries"
}
REV="master"
rm -f nix-pkgs.json
nix-shell --run "cargo run --release -- -s ${REV} -o unstable.json"
only_cve < unstable.json > unstable-cve.json

REV="release-18.03"
rm -f nix-pkgs.json
nix-shell --run "cargo run --release -- -s ${REV} -o stable.json"
only_cve < stable.json > stable-cve.json
