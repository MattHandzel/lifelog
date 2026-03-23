{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.lifelog.collector;
  toml = pkgs.formats.toml {};
  configFile = toml.generate "lifelog-collector.toml" cfg.settings;
in {
  options.services.lifelog.collector = {
    enable = lib.mkEnableOption "Lifelog collector";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The lifelog-collector package to use.";
    };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = toml.type;
        options = {
          runtime = lib.mkOption {
            type = lib.types.submodule {
              freeformType = toml.type;
              options = {
                collectorId = lib.mkOption {
                  type = lib.types.str;
                  description = "Unique identifier for this collector instance.";
                };
              };
            };
            default = {};
            description = "Runtime configuration (maps to [runtime] in TOML).";
          };
          server = lib.mkOption {
            type = lib.types.submodule {
              freeformType = toml.type;
              options = {
                host = lib.mkOption {
                  type = lib.types.str;
                  default = "127.0.0.1";
                  description = "Address of the Lifelog server to connect to.";
                };
                port = lib.mkOption {
                  type = lib.types.port;
                  default = 7182;
                  description = "Port of the Lifelog server.";
                };
              };
            };
            default = {};
            description = "Server connection configuration (maps to [server] in TOML).";
          };
        };
      };
      default = {};
      description = ''
        Configuration for the Lifelog collector, serialized to TOML.
        See docs/CONFIG.md for all available options.
      '';
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "%h/lifelog/data";
      description = "Base directory for collector data. Supports %h for home directory.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.lifelog-collector = {
      description = "Lifelog Collector";
      wantedBy = ["graphical-session.target"];
      after = ["graphical-session.target"];
      partOf = ["graphical-session.target"];

      restartTriggers = [configFile];

      environment = {
        LIFELOG_CONFIG = configFile;
      };

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package}";
        Restart = "on-failure";
        RestartSec = 10;
        StartLimitBurst = 5;

        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ReadWritePaths = [
          cfg.dataDir
          "%h/.config/lifelog"
        ];
        RestrictSUIDSGID = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = ["@system-service" "~@privileged"];
      };
    };
  };
}
