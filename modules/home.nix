{ config, lib, pkgs, ... }:

with lib;

let

  cfg = config.programs.npkg;

  jsonFormat = pkgs.formats.json { };

in

{
  options = {
    programs.npkg = {
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

  config = mkIf (cfg.systemconfig != "/etc/nixos/configuration.nix" || cfg.homeconfig != "${config.home.homeDirectory}/.config/nixpkgs/home.nix" || cfg.flake != null) {
    xdg.configFile."npkg/config.json".source = jsonFormat.generate "config.json" cfg;
  };
}
