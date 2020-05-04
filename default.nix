{
  pkgsSrc ? builtins.fetchTarball {
    url = https://github.com/NixOS/nixpkgs/archive/fce7562cf46727fdaf801b232116bc9ce0512049.tar.gz;
    sha256 = "14rvi69ji61x3z88vbn17rg5vxrnw2wbnanxb7y0qzyqrj7spapx";
  }
}:
let
  pkgs = import pkgsSrc {
    overlays = [
      (self: super: {
        defaultCrateOverrides = super.defaultCrateOverrides // {
          libsqlite3-sys = attrs: {
            buildInputs = with self; [ pkgconfig sqlite ];
            postConfigure = ''
              sed -i 's/-/_/g' target/env
            '';
          };
        };
      })
    ];
  };
  crates = pkgs.callPackage ./nix/crate2nix.nix {};
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

  inherit crates;
  nix-vuln-scanner = crates.rootCrate.build.overrideAttrs (_: {
    preBuild = ''
      rm -rf static
      ln -s ${staticFiles} static
    '';
  });

  shell = let
    pkgs = import pkgsSrc {
      overlays = [
        (_: super: {
          crate2nix = super.callPackage (builtins.fetchTarball {
            url = https://github.com/kolloch/crate2nix/archive/9d027d8cb6d566861f2f0e4ca905fd2e32fb8841.tar.gz;
            sha256 = "1gkwvlrjcblbyfdjr1l8zbcni62zc01xii2yra61vrmphbwxnczf";
          }) {};
        })
      ];
    };
  in pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ carnix pkgconfig libssh2 cmake gitAndTools.pre-commit brotli nix-prefetch-git cargo rustc crate2nix ];
    buildInputs = with pkgs; [ sqlite openssl zlib mysql postgresql ];
    meta.hydraPlatforms = [];
  };

  gcroot = pkgs.symlinkJoin {
    name = "nix-vuln-scanner-gcroot";
    paths = (map pkgs.lib.getLib shell.buildInputs) ++ shell.nativeBuildInputs ++ [ pkgsSrc ];
  };
}
