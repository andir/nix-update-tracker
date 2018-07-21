with import <nixpkgs> {};
let
  pkg = callPackage ./default.nix {};
in
  pkg.overrideAttrs (old: {
    nativeBuildInputs = (old.nativeBuildInputs or []) ++ [ cmake pkgconfig ];
    buildInputs = (old.buildInputs or []) ++ [ openssl ];
    shellHook = ''
      export PATH="$HOME/.cargo/bin:$PATH"
    '';
  })
#pkgs.mkShell {
#  buildInputs = with pkgs; [
#    rustc
#    cargo
#    pkgconfig
#    openssl.dev
#    linuxPackages.perf
#  ];
#  shellHook = ''
#    export PATH="$HOME/.cargo/bin:$PATH"
#    #   exec ${pkgs.zsh}/bin/zsh
#  '';
#}
