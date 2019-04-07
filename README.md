# Nix Vulnerability Scanner

Provides information aobut the current and past vulnerabilities within NixOS/Nixpkgs channels.

My instance of this is available at [broken.sh](https://broken.sh).

# Building

You can just build the attrtibute `nix-vuln-scanner` to build the actual project. My hydra usually publishes binary artifacts fro nixos unstable and the currently pinned checkout. You can use the below command to use the binary cache.

```
nix-build -A nix-vuln-scanner \
	--option substituters "https://cache.nixos.org/ https://andir-nixpkgs.ams3.cdn.digitaloceanspaces.com/" \
	--option trusted-public-keys "zeta:9zm3cHRlqz3T9HnRsodtQGGqHOLDAiB+8d0kOKnFI0M= cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
```

# Development

For development I recommend using `nix-shell` or `direnv`. Compilation can be done through `cargo build` as usual.

The web component of the project requires a few static files to be present in `static/`. The files are managed with Nix and during a normal build they'll be provided to the package. To build it manually (using cargo/rustc) you will have to copy them to `static/` manually like below.

```
cp -rv $(nix-build -A staticFiles --no-out-link)/* static/
```
