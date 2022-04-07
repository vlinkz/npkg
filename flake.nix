{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem
      (system:
       let 
          name = "npkg";
          pkgs = import nixpkgs { inherit system; };
          naersk-lib = naersk.lib."${system}";
        in rec {
          packages.${name} = naersk-lib.buildPackage {
            pname = "${name}";
            root = ./.;
            copyLibs = true;
            buildInputs = with pkgs; [
              openssl
              pkgconfig
            ];
          };

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
            nativeBuildInputs = 
              with pkgs; [ rustc cargo openssl pkgconfig ] ;
          };
        }
      );
}
