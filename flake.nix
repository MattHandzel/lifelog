{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    systems = ["x86_64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
  in {
    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      commonDeps = {
        buildInputs = with pkgs;
          [
            pkg-config
            openssl
            sqlite
          ]
          ++ pkgs.lib.optionals (pkgs.stdenv.isLinux) [
            alsa-lib
            linuxPackages.v4l2loopback
            v4l-utils
            xorg.libX11
            xorg.libXtst
            xorg.libXi
          ];

        # TODO: Work out dependencies for front end and the rest of the software
        frontendDeps = with pkgs; [
          gtk3
          atk
          glib
          webkitgtk
          libsoup
        ];

        nativeBuildInputs = with pkgs;
          [
            pkg-config
            cmake
          ]
          ++ pkgs.lib.optionals (pkgs.stdenv.isLinux) [
            alsa-lib
          ];
      };

      mkRustPackage = {
        pname,
        binName,
      }:
        pkgs.rustPlatform.buildRustPackage {
          inherit pname;
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = ["-p" binName];

          buildInputs = commonDeps.buildInputs;
          nativeBuildInputs = commonDeps.nativeBuildInputs;

          meta = with pkgs.lib; {
            description = "A project to log various sources of data for your lifelog";
            license = licenses.mit;
            maintainers = [maintainers.MattHandzel];
          };
        };
    in {
      lifelog-server = mkRustPackage {
        pname = "lifelog-server";
        binName = "lifelog-server";
      };
      lifelog-collector = mkRustPackage {
        pname = "lifelog-collector";
        binName = "lifelog-collector";
      };
    });

    devShells = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
    in {
      default = pkgs.mkShell {
        PKG_CONFIG_PATH =
          pkgs.lib.optionalString pkgs.stdenv.isLinux
          "${pkgs.alsa-lib}/lib/pkgconfig";

        packages = with pkgs;
          [
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src" "rust-analyzer"];
            })
            openssl
            sqlite
            pkg-config
            cmake
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            linuxPackages.v4l2loopback
            gtk3
            v4l-utils
            libxkbcommon
            alsa-lib
            grim
            xorg.libX11
            slurp
            xorg.libXtst
            xorg.libXi

            glib
            gtk3
            atk
          ];
      };
    });
  };
}
