(
        let
          pkgs = import <nixpkgs> {};

          getPkg = path: assert path != []; pkgs.lib.attrByPath path {} pkgs;
          getPatches = pkg: pkg.patches or [];
          asList = p: let
            type = builtins.typeOf p;
          in pkgs.lib.flatten (if type == "path" then [p] else if type == "string" then [p] else p);
          getPatchName = patch: if builtins.typeOf patch == "path" then toString patch else patch.name;
        in
          map getPatchName (asList (getPatches (getPkg __pkg_path__)))
)

