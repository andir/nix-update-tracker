{ pkgs ? import <nixos-unstable> {}}:
let
  crates = (pkgs.callPackage ./crates.nix {});
  drv = crates.nix_vuln_scanner {};
in drv.override {
  crateOverrides = pkgs.defaultCrateOverrides // {
    libssh2-sys = attrs: {
      nativeBuildInputs = [ pkgs.cmake ];
      buildInputs = attrs.buildInputs or [] ++ [ pkgs.zlib pkgs.openssl ];
    };
  };
}
