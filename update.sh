#!/bin/sh

function only_cve() {
  jq "[to_entries[] | select(.value.CVEs != [])]|from_entries"
}

nix-update-tracker -u -s https://github.com/nixos/nixpkgs/archives/master.tar.gz -o unstable.json
only_cve < unstable.json > unstable-cve.json

nix-update-tracker -u -s https://github.com/nixos/nixpkgs/archives/nixos-17.09.tar.gz -o stable.json
only_cve < stable.json > stable-cve.json
