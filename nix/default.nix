{ pkgs }:
rec {
  carnix = pkgs.callPackage ./carnix.nix {};
  diesel = pkgs.callPackage ./diesel.nix {};
}
