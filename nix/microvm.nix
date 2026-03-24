{
  self,
  pkgs,
  lib,
  ...
}: let
  serverPkg = self.packages.${pkgs.system}.lifelog-server;
  collectorPkg = self.packages.${pkgs.system}.lifelog-collector;
in {
  imports = [
    self.nixosModules.lifelog-server
    self.nixosModules.lifelog-collector
    self.nixosModules.lifelog-postgres
  ];

  microvm = {
    hypervisor = "cloud-hypervisor";
    vcpu = 4;
    mem = 4096;

    shares = [
      {
        tag = "nix-store";
        source = "/nix/store";
        mountPoint = "/nix/.ro-store";
        proto = "virtiofs";
      }
      {
        tag = "project";
        source = "/home/matth/Projects/lifelog";
        mountPoint = "/workspace";
        proto = "virtiofs";
      }
    ];

    volumes = [
      {
        image = "vm-state.img";
        mountPoint = "/var";
        size = 2048;
      }
    ];

    interfaces = [
      {
        type = "user";
        id = "vm-lifelog";
      }
    ];
  };

  networking = {
    hostName = "lifelog-test-vm";
    firewall.enable = false;
  };

  users.users.test = {
    isNormalUser = true;
    group = "test";
    home = "/home/test";
  };
  users.groups.test = {};

  # --- V2: Virtual display (Xvfb) ---
  environment.systemPackages = with pkgs; [
    xorg-server
    xauth
    tesseract
    leptonica
    coreutils
    bash
  ];

  systemd.services.xvfb = {
    description = "Xvfb virtual framebuffer";
    wantedBy = ["multi-user.target"];
    before = ["lifelog-collector.service"];

    serviceConfig = {
      ExecStart = "${pkgs.xorg-server}/bin/Xvfb :99 -screen 0 1920x1080x24 -ac +extension GLX +render -noreset";
      Restart = "on-failure";
      RestartSec = 2;
      Type = "simple";
    };
  };

  environment.variables.DISPLAY = ":99";

  # --- V3: Basic services ---
  services.lifelog.postgres = {
    enable = true;
  };

  services.postgresql.package = lib.mkForce pkgs.postgresql_16;

  services.lifelog.server = {
    enable = true;
    package = serverPkg;
    settings = {
      server = {
        host = "0.0.0.0";
        port = 7182;
        casPath = "/var/lib/lifelog/cas";
        serverName = "LifelogTestVM";
      };
    };
    environmentFile = "/var/lib/lifelog/env";
  };

  services.lifelog.collector = {
    enable = true;
    package = collectorPkg;
    settings = {
      runtime = {
        collectorId = "test-vm-collector";
      };
      server = {
        host = "127.0.0.1";
        port = 7182;
      };
    };
    dataDir = "/home/test/lifelog/data";
  };

  # --- V4: Networking ---
  # The "user" interface type provides NAT via SLIRP.
  # VM can reach the host and internet. Host reaches VM via port forwards.

  systemd.services.lifelog-env-setup = {
    description = "Create lifelog environment file for server";
    wantedBy = ["multi-user.target"];
    before = ["lifelog-server.service"];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };
    script = ''
      mkdir -p /var/lib/lifelog
      cat > /var/lib/lifelog/env <<'EOF'
      LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@localhost/lifelog
      EOF
    '';
  };

  # Auto-login for test automation
  services.getty.autologinUser = "test";

  system.stateVersion = "24.11";
}
