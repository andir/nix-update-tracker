{ nativePkgs ? import <nixpkgs> {}}:
let
  pkgs = import (nativePkgs.fetchgit {
    url = "https://github.com/nixos/nixpkgs";
    rev = "f83a26291fcef25609796110f0cdd76b7d02942a";
    sha256 = "1xivb9jpzlhp78am2r68fs0g9x4jryf06pv1niw05zv7664j5f2d";
    fetchSubmodules = true;
  }) {};
  crates = pkgs.callPackage ./Cargo.nix {};
  drv = crates.nix_vuln_scanner {
    no-stdlib = false;
  };
in rec {
  nix-vuln-scanner = drv.override {
    crateOverrides = pkgs.defaultCrateOverrides // {
      libssh2-sys = attrs: {
        nativeBuildInputs = [ pkgs.cmake ];
        buildInputs = attrs.buildInputs or [] ++ [ pkgs.zlib pkgs.openssl ];
      };
      alloc_no_stdlib = attrs: {
        crateBin = [];
      };
    };
    preBuild = ''
      ${pkgs.pandoc}/bin/pandoc -f markdown -t html -s static/index.md > static/index.html
    '';
  };

  carnix = (pkgs.callPackage ./nix {}).carnix;
  shell = pkgs.mkShell {
    nativeBuildInputs = [ carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.pandoc pkgs.gitAndTools.pre-commit pkgs.brotli pkgs.nix-prefetch-git ];
    buildInputs = [ pkgs.sqlite pkgs.openssl pkgs.zlib ];
  };
}
