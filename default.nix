{ pkgs ? import <nixos-unstable> {}}:
(pkgs.callPackage ./crates.nix {}).nix_vuln_scanner {}
