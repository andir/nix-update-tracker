{pkgs}:
let
  crates_nix = ./diesel-cargo.nix;
  orig_src = pkgs.fetchFromGitHub {
    owner = "diesel-rs";
    repo = "diesel";
    rev = "a61646c7ae762e90f8546201529f150d343a7df0";
    sha256 = "0gqqx78k83v75b2f706gd7bhg0l7cmwaslwfxv177yik8fgx39px";
  };
  patched_src = pkgs.runCommand "patched-diesel-src" {} ''
    cp -r ${orig_src} $out
    chmod +w -R $out
    cp ${crates_nix} $out/default.nix
  '';
  crates = (pkgs.callPackage "${patched_src}/default.nix" {});
in (crates.diesel_cli {
  mysql = false;
  postgres = false;
  sqlite = true;
}).override {
  crateOverrides = pkgs.defaultCrateOverrides // {
    diesel_infer_schema = attrs: {
      features = [ "sqlite" "postgres" "mysql" "lint" ];
    };
    diesel = attrs: {
      features = [ "sqlite" "postgres" "mysql" "extras" ];
      buildInputs = with pkgs; [ postgresql mysql sqlite ];
    };
  };
}
