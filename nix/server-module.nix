{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.lifelog.server;
  toml = pkgs.formats.toml {};
  configFile = toml.generate "lifelog-config.toml" cfg.settings;
in {
  options.services.lifelog.server = {
    enable = lib.mkEnableOption "Lifelog server";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The lifelog-server package to use.";
    };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = toml.type;
        options = {
          server = lib.mkOption {
            type = lib.types.submodule {
              freeformType = toml.type;
              options = {
                host = lib.mkOption {
                  type = lib.types.str;
                  default = "127.0.0.1";
                  description = "Address the server listens on.";
                };
                port = lib.mkOption {
                  type = lib.types.port;
                  default = 7182;
                  description = "Port the server listens on.";
                };
                casPath = lib.mkOption {
                  type = lib.types.str;
                  default = "/var/lib/lifelog/cas";
                  description = "Path to the content-addressable store.";
                };
                serverName = lib.mkOption {
                  type = lib.types.str;
                  default = "LifelogServer";
                  description = "Display name for the server instance.";
                };
                defaultCorrelationWindowMs = lib.mkOption {
                  type = lib.types.int;
                  default = 30000;
                  description = "Default time window (ms) for correlating events across modalities.";
                };
              };
            };
            default = {};
            description = "Server configuration (maps to [server] in TOML).";
          };
        };
      };
      default = {};
      description = ''
        Configuration for the Lifelog server, serialized to TOML.
        See docs/CONFIG.md for all available options.
      '';
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Path to an environment file containing secrets.
        Used for LIFELOG_TLS_CERT_PATH, LIFELOG_TLS_KEY_PATH,
        LIFELOG_POSTGRES_INGEST_URL, etc.
      '';
    };

    tlsCertFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to the TLS certificate file. Must not be in the Nix store.";
    };

    tlsKeyFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = "Path to the TLS private key file. Must not be in the Nix store.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Whether to open the server port in the firewall.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.tlsCertFile == null || !lib.hasPrefix "/nix/store" (toString cfg.tlsCertFile);
        message = "services.lifelog.server.tlsCertFile points to the Nix store, which is world-readable. Use a quoted absolute path instead.";
      }
      {
        assertion = cfg.tlsKeyFile == null || !lib.hasPrefix "/nix/store" (toString cfg.tlsKeyFile);
        message = "services.lifelog.server.tlsKeyFile points to the Nix store, which is world-readable. Use a quoted absolute path instead.";
      }
      {
        assertion = (cfg.tlsCertFile == null) == (cfg.tlsKeyFile == null);
        message = "services.lifelog.server: tlsCertFile and tlsKeyFile must both be set or both be null.";
      }
    ];

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [cfg.settings.server.port];

    systemd.services.lifelog-server = {
      description = "Lifelog Server";
      wantedBy = ["multi-user.target"];
      after = ["network-online.target" "postgresql.service"];
      requires = ["network-online.target"];
      wants = ["postgresql.service"];

      restartTriggers = [configFile];

      environment = {
        LIFELOG_CONFIG = configFile;
      }
      // lib.optionalAttrs (cfg.tlsCertFile != null) {
        LIFELOG_TLS_CERT_PATH = toString cfg.tlsCertFile;
        LIFELOG_TLS_KEY_PATH = toString cfg.tlsKeyFile;
      };

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package}";
        Restart = "on-failure";
        RestartSec = 10;
        StartLimitBurst = 5;

        DynamicUser = true;
        StateDirectory = "lifelog";
        StateDirectoryMode = "0700";
        RuntimeDirectory = "lifelog";
        RuntimeDirectoryMode = "0700";

        EnvironmentFile = lib.optional (cfg.environmentFile != null) cfg.environmentFile;

        BindReadOnlyPaths =
          lib.optional (cfg.tlsCertFile != null) (toString cfg.tlsCertFile)
          ++ lib.optional (cfg.tlsKeyFile != null) (toString cfg.tlsKeyFile);

        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        ProtectHome = true;
        ProtectSystem = "strict";
        RestrictSUIDSGID = true;

        CapabilityBoundingSet = "";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = ["@system-service" "~@privileged"];
        UMask = "0077";
      };
    };
  };
}
