{ ... }@args:
{
  inherit (import ./default.nix args)
    nix-vuln-scanner
    staticFiles
    gcroot;
}
