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
        type = with types; nullOr path;
        default = null;
        example = literalExpression ''/home/user/.config/nixpkgs/home.nix'';
        description = ''Where npkg looks for home.nix'';
      };
      flake = mkOption {
        type = with types; nullOr path;
        default = null;
        example = literalExpression ''/etc/nixos/flake.nix'';
        description = ''Where npkg looks for flake.nix'';
      };
    };
  };
  
  config = mkIf (cfg.systemconfig != "/etc/nixos/configuration.nix" || cfg.homeconfig != null || cfg.flake != null) {
    environment.etc."npkg/config.json".source = jsonFormat.generate "config.json" cfg;
  };
}
