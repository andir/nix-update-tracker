with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "rust-env";
  src = null;
  buildInputs = with pkgs; [
    pkgconfig
    openssl.dev
  ];
}
