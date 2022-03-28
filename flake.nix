{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:balsoft/crate2nix/tools-nix-version-comparison";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, crate2nix, ... }:
    utils.lib.eachDefaultSystem
      (system:
       let 
          name = "npkg";
          pkgs = import nixpkgs { inherit system; };
          inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
            generatedCargoNix;
          project = pkgs.callPackage (generatedCargoNix {
            inherit name;
            src = ./.;
          }) {};
        in rec {
          packages.${name} = project.rootCrate.build;

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = apps.${name};

          # `nix develop`
          devShell = pkgs.mkShell {
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            nativeBuildInputs = 
              with pkgs; [ rustc cargo pkgconfig openssl.dev ] ;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );
}