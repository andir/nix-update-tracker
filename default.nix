{
  pkgsSrc ? builtins.fetchTarball {
    url = https://github.com/NixOS/nixpkgs/archive/7815c86c104a99417db844791dcda34fe7a7965f.tar.gz;
    sha256 = "0k6ws2b2b6vrvq2g5h8fi8qscb0wk0wy097cnf36f9acd126k43j";
  }
}:
let
  pkgs = import pkgsSrc {};
  cratesIO = pkgs.callPackage ./crates-io.nix {};
  crates = pkgs.callPackage ./Cargo.nix { inherit cratesIO; };
  drv = crates.nix_vuln_scanner {};
  _shell = {
    nativeBuildInputs = with pkgs; [ carnix pkgconfig libssh2 cmake gitAndTools.pre-commit brotli nix-prefetch-git cargo rustc ];
    buildInputs = with pkgs; [ sqlite openssl zlib mysql postgresql ];
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
    (pkgs.fetchurl {
      name = "github.png";
      url = https://github.com/andir/ipv6.watch/raw/672870bb1bdbb929f44ee84884192ddc6eddc479/dist/images/github.png;
      sha256 = "0dw9m2vshjm3jpn8nxac88k3nyq7yj52hq3br0kq9vjcfzzrbv92";
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
