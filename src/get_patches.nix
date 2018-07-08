(
        let
          pkgs = import <nixpkgs> {};
          pkgName = "__pkg_name__";

          getPkg = name: if (builtins.hasAttr name pkgs) then pkgs."${name}" else {};
          getPatches = pkg: (if (builtins.hasAttr "patches" pkg) then pkg.patches else []);
          asList = p: let
            type = builtins.typeOf p;
          in pkgs.lib.flatten (if type == "path" then [p] else if type == "string" then [p] else p);
          getPatchName = patch: if builtins.typeOf patch == "path" then "${patch}" else patch.name;
        in
          map getPatchName (asList (getPatches (getPkg pkgName)))
)

