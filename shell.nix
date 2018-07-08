with import <nixpkgs> {};
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkgconfig
    openssl.dev
  ];
  shellHook = ''
    export PATH="$HOME/.cargo/bin:$PATH"
    #   exec ${pkgs.zsh}/bin/zsh
  '';
}
