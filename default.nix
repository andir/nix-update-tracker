{ pkgs ? import <nixos-unstable> {}}:
let
  crates = (pkgs.callPackage ./Cargo.nix {});
  drv = crates.nix_vuln_scanner {};
in rec {
  nix-vuln-scanner = drv.override {
      crateOverrides = pkgs.defaultCrateOverrides // {
        libssh2-sys = attrs: {
        nativeBuildInputs = [ pkgs.cmake ];
        buildInputs = attrs.buildInputs or [] ++ [ pkgs.zlib pkgs.openssl ];
      };
    };
    preBuild = ''
      ${pkgs.pandoc}/bin/pandoc -f markdown -t html -s static/index.md > static/index.html
    '';
  };

  carnix = (pkgs.callPackage ./nix {}).carnix;
  shell = pkgs.mkShell {
    nativeBuildInputs = [ carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.pandoc pkgs.gitAndTools.pre-commit ];
    buildInputs = [ pkgs.sqlite pkgs.openssl pkgs.zlib ];
  };
}
