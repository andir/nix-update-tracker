{ nativePkgs ? import <nixpkgs> {}
, pkgsSrc ? builtins.fetchTarball {
    url = https://github.com/NixOS/nixpkgs-channels/archive/36f316007494c388df1fec434c1e658542e3c3cc.tar.gz;
    sha256 = "1w1dg9ankgi59r2mh0jilccz5c4gv30a6q1k6kv2sn8vfjazwp9k";
  }
}:
let
  pkgs = import pkgsSrc {};
  cratesIO = pkgs.callPackage ./crates-io.nix {};
  crates = pkgs.callPackage ./Cargo.nix { inherit cratesIO; };
  drv = crates.nix_vuln_scanner {};
  shell = {
    nativeBuildInputs = [ pkgs.carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.pandoc pkgs.gitAndTools.pre-commit pkgs.brotli pkgs.nix-prefetch-git (pkgs.callPackage ./nix/carnix.nix {}) ];
    buildInputs = [ pkgs.sqlite pkgs.openssl pkgs.zlib pkgs.mysql pkgs.postgresql ];
  };

in {
  nix-vuln-scanner = drv.override {
    crateOverrides = let
      crateOverrides = (pkgs.defaultCrateOverrides // {
        libssh2-sys = attrs: {
          nativeBuildInputs = [ pkgs.cmake ];
          buildInputs = attrs.buildInputs or [] ++ [ pkgs.zlib pkgs.openssl ];
        };
        alloc_no_stdlib = attrs: {
          crateBin = [];
        };
      });

      updateOverrides = name: new: let
        oldOverrides = (pkgs.defaultCrateOverrides.${name} or (attrs: {}));
      in attrs: (oldOverrides attrs) // new;

      buildOverrides = pkgs.lib.listToAttrs (map (name: pkgs.lib.nameValuePair name (updateOverrides name { extraRustcOpts = ["--edition 2018"]; })) (pkgs.lib.attrNames cratesIO.crates));
      in crateOverrides;
    preBuild = ''
      ${pkgs.pandoc}/bin/pandoc -f markdown -t html -s static/index.md > static/index.html
    '';
  };

  shell = pkgs.mkShell shell;

  gcroot = pkgs.symlinkJoin {
    name = "nix-vuln-scanner-gcroot";
    paths = (map pkgs.lib.getLib shell.buildInputs) ++ shell.nativeBuildInputs ++ [ pkgsSrc ];
  };
}
