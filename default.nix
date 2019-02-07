{ nativePkgs ? import <nixpkgs> {}}:
let
  pkgs = import (builtins.fetchTarball {
    url = https://github.com/NixOS/nixpkgs-channels/archive/2d6f84c1090ae39c58dcec8f35a3ca62a43ad38c.tar.gz;
    sha256 = "0l8b51lwxlqc3h6gy59mbz8bsvgc0q6b3gf7p3ib1icvpmwqm773";
  }) {};
  cratesIO = pkgs.callPackage ./crates-io.nix {};
  crates = pkgs.callPackage ./Cargo.nix { inherit cratesIO; };
  drv = crates.nix_vuln_scanner {
  #  no-stdlib = false;
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

  carnix = pkgs.carnix;
  shell = pkgs.mkShell {
    nativeBuildInputs = [ carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.pandoc pkgs.gitAndTools.pre-commit pkgs.brotli pkgs.nix-prefetch-git ];
    buildInputs = [ pkgs.sqlite pkgs.openssl pkgs.zlib ];
  };
}
