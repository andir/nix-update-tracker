{ pkgsSrc ? null, ... }@_args:
let
  args = if pkgsSrc == null then removeAttrs _args ["pkgsSrc"] else _args;
in
{
  inherit (import ./default.nix args)
    nix-vuln-scanner
    staticFiles
    gcroot;
}
