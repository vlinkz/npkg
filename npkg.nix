{ config, lib, pkgs, ... }:

with lib;

let

  npkg-pkg = with pkgs; import (
    fetchFromGitHub {
      owner = "vlinkz";
      repo = "npkg";
      rev = "0.0.10";
      sha256 = "YtQXsu4faBlphtjKPaG4Si099bHVidAUtPSGNfPcqas=";
    }
  );

  cfg = config.programs.npkg;

  jsonFormat = pkgs.formats.json { };

in

{
  options = {

    programs.npkg = {

      enable = mkEnableOption "npkg package management wrapper";

      settings = {
        systemconfig = mkOption {
          type = types.path;
          default = "/etc/nixos/configuration.nix";
          example = literalExpression ''/home/user/nix/configuration.nix'';
          description = ''Where npkg looks for configuration.nix'';
        };
        homeconfig = mkOption {
          type = types.path;
          default = "${config.home.homeDirectory}/.config/nixpkgs/home.nix";
          example = literalExpression ''/home/user/nix/home.nix'';
          description = ''Where npkg looks for home.nix'';
        };
        flake = mkOption {
          type = with types; nullOr path;
          default = null;
          example = literalExpression ''/home/user/nix/flake.nix'';
          description = ''Where npkg looks for flake.nix'';
        };
      };
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ npkg-pkg.packages.x86_64-linux.npkg ];
    xdg.configFile."npkg/config.json".source = jsonFormat.generate "config.json" cfg.settings;
  };
}
