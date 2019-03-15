{
  pkgsSrc ? builtins.fetchTarball {
    url = https://github.com/andir/nixpkgs/archive/6a3b144d02d8b1215142be610ec9b8c1f2213094.tar.gz;
    sha256 = "1ydwwh7az5q1cvndazl2vr8flyk1lhqc7qyyhx946kn3jczmc2dy";
  }
}:
let
  pkgs = import pkgsSrc {};
  cratesIO = pkgs.callPackage ./crates-io.nix {};
  crates = pkgs.callPackage ./Cargo.nix { inherit cratesIO; };
  drv = crates.nix_vuln_scanner {};
  shell = {
    nativeBuildInputs = [ pkgs.carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.pandoc pkgs.gitAndTools.pre-commit pkgs.brotli pkgs.nix-prefetch-git ];
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
