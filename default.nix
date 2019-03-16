{
  pkgsSrc ? builtins.fetchTarball {
    url = https://github.com/NixOS/nixpkgs/archive/36d6bbaa70b0427e4923f20831021114a31a7022.tar.gz;
    sha256 = "19qw2728ww2cpmi6j9a9q4iwfflgdsai5mri973lg12c2sjv5z4w";
  }
}:
let
  pkgs = import pkgsSrc {};
  cratesIO = pkgs.callPackage ./crates-io.nix {};
  crates = pkgs.callPackage ./Cargo.nix { inherit cratesIO; };
  drv = crates.nix_vuln_scanner {};
  _shell = {
    nativeBuildInputs = [ pkgs.carnix pkgs.pkgconfig pkgs.libssh2 pkgs.cmake pkgs.gitAndTools.pre-commit pkgs.brotli pkgs.nix-prefetch-git ];
    buildInputs = [ pkgs.sqlite pkgs.openssl pkgs.zlib pkgs.mysql pkgs.postgresql ];
  };

in rec {
  staticAssets = [
    (pkgs.fetchurl {
      name = "popper-1.14.7.min.js";
      url = https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.14.7/umd/popper.min.js;
      sha256 = "1hhi8jw0gv2f1ir59ydzvgrspj6vb121wf36ddl4mdm93xza1wv6";
    })
    (pkgs.fetchurl {
      name = "jquery-3.3.1.slim.min.js";
      url = https://code.jquery.com/jquery-3.3.1.slim.min.js;
      sha256 = "1cgv6agm53y854kpvslcd1xwjf63m350c7y8gywhxlwh5fdnpryx";
    })
    (pkgs.fetchzip {
      name = "bootstrap-4.3.1";
      url = https://github.com/twbs/bootstrap/releases/download/v4.3.1/bootstrap-4.3.1-dist.zip;
      sha256 = "1j2103jl1d99y7i7nlzr54b8w54mlgdca6mqxwqdxvis5kaqzzri";
    })
  ];

  staticFiles = pkgs.runCommand "static-files"  {} ''
    mkdir $out
    ${pkgs.lib.concatMapStringsSep "\n" (src: "ln -s ${src} $out/${src.name}") staticAssets}
  '';

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
      rm -rf static
      ln -s ${staticFiles} static
    '';
  };

  shell = pkgs.mkShell _shell // { meta.hydraPlatforms = []; };

  gcroot = pkgs.symlinkJoin {
    name = "nix-vuln-scanner-gcroot";
    paths = (map pkgs.lib.getLib shell.buildInputs) ++ shell.nativeBuildInputs ++ [ pkgsSrc ];
  };
}
