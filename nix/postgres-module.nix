{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.lifelog.postgres;
in {
  options.services.lifelog.postgres = {
    enable = lib.mkEnableOption "PostgreSQL provisioning for Lifelog";
    database = lib.mkOption {
      type = lib.types.str;
      default = "lifelog";
      description = "PostgreSQL database name for Lifelog.";
    };
    user = lib.mkOption {
      type = lib.types.str;
      default = "lifelog";
      description = "PostgreSQL role for Lifelog.";
    };
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.postgresql_16;
      description = "PostgreSQL package used by the managed service.";
    };
  };

  config = lib.mkIf cfg.enable {
    services.postgresql = {
      enable = true;
      package = lib.mkDefault cfg.package;
      ensureDatabases = [cfg.database];
      ensureUsers = [
        {
          name = cfg.user;
          ensureDBOwnership = true;
        }
      ];
    };
  };
}
