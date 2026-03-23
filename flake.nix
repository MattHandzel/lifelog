{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    systems = ["x86_64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs systems;
  in {
    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      commonDeps = {
        buildInputs = with pkgs;
          [
            pkg-config
            openssl
            sqlite
            protobuf
            rocksdb
          ]
          ++ pkgs.lib.optionals (pkgs.stdenv.isLinux) [
            alsa-lib
            linuxPackages.v4l2loopback
            v4l-utils
            libx11
            libxtst
            libxi
            leptonica
            tesseract
          ];

        # TODO: Work out dependencies for front end and the rest of the software
        frontendDeps = with pkgs; [
          gtk3
          atk
          glib
          webkitgtk_4_1
          libsoup_3
        ];

        nativeBuildInputs = with pkgs;
          [
            llvmPackages.clang
            pkg-config
            cmake
          ]
          ++ pkgs.lib.optionals (pkgs.stdenv.isLinux) [
            alsa-lib

            protobuf # we need this b/c we need to build the proto files
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

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang}/lib";

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
      };
    in {
      default = pkgs.mkShell {
        PKG_CONFIG_PATH =
          pkgs.lib.optionalString pkgs.stdenv.isLinux
          "${pkgs.alsa-lib}/lib/pkgconfig";

        # Dev builds produced by `cargo build/test` are not patched with Nix RPATHs.
        # Set LD_LIBRARY_PATH so running workspace binaries (and subprocesses spawned by tests)
        # can find required system libraries like X11 and ALSA.
        LD_LIBRARY_PATH =
          pkgs.lib.optionalString pkgs.stdenv.isLinux
          (pkgs.lib.makeLibraryPath [
            pkgs.alsa-lib
            pkgs.libv4l
            pkgs.libxkbcommon
            pkgs.libx11
            pkgs.libxi
            pkgs.libxtst
            pkgs.libdrm
            pkgs.libglvnd
            pkgs.mesa
            pkgs.vulkan-loader
            pkgs.webkitgtk_4_1
            pkgs.libsoup_3
            pkgs.gtk3
            pkgs.glib
            pkgs.pango
            pkgs.cairo
            pkgs.gdk-pixbuf
            pkgs.atk
            pkgs.stdenv.cc.cc.lib
          ]);

        LIBGL_DRIVERS_PATH =
          pkgs.lib.optionalString pkgs.stdenv.isLinux
          "${pkgs.mesa}/lib/dri";
        GBM_BACKENDS_PATH =
          pkgs.lib.optionalString pkgs.stdenv.isLinux
          "${pkgs.mesa}/lib/gbm";

        RUSTC_WRAPPER = "sccache";

        packages = with pkgs;
          [
            rustc
            cargo
            rust-analyzer
            rustfmt
            openssl
            sqlite
            protobuf
            pkg-config
            cmake
            sccache
            cargo-nextest
            bacon
            sqlx-cli
            postgresql
            cargo-tauri
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            linuxPackages.v4l2loopback
            gtk3
            v4l-utils
            libxkbcommon
            alsa-lib
            grim
            libx11
            slurp
            libxtst
            libxi

            glib
            gtk3
            atk
            webkitgtk_4_1
            libsoup_3

            leptonica
            tesseract
          ];

        # Tesseract needs these at runtime
        TESSDATA_PREFIX = "${pkgs.tesseract}/share/tessdata";
      };

      frontend = pkgs.mkShell {
        PKG_CONFIG_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux (
          pkgs.lib.concatStringsSep ":" [
            "${pkgs.alsa-lib}/lib/pkgconfig"
            "${pkgs.webkitgtk_4_1}/lib/pkgconfig"
            "${pkgs.libsoup_3}/lib/pkgconfig"
            "${pkgs.gtk3}/lib/pkgconfig"
            "${pkgs.glib.dev}/lib/pkgconfig"
            "${pkgs.openssl.dev}/lib/pkgconfig"
          ]
        );

        LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux (
          pkgs.lib.makeLibraryPath [
            pkgs.webkitgtk_4_1
            pkgs.libsoup_3
            pkgs.gtk3
            pkgs.glib
            pkgs.pango
            pkgs.cairo
            pkgs.gdk-pixbuf
            pkgs.harfbuzz
            pkgs.alsa-lib
            pkgs.libv4l
            pkgs.libxkbcommon
            pkgs.libx11
            pkgs.libxi
            pkgs.libxtst
            pkgs.openssl
            pkgs.stdenv.cc.cc.lib
          ]
        );

        RUSTC_WRAPPER = "sccache";
        TESSDATA_PREFIX = "${pkgs.tesseract}/share/tessdata";

        packages = with pkgs;
          [
            rustc
            cargo
            rust-analyzer
            rustfmt
            openssl
            sqlite
            pkg-config
            cmake
            sccache
            cargo-nextest
            nodejs_22
            nodePackages.npm
            cargo-tauri
            protobuf
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            webkitgtk_4_1
            libsoup_3
            gtk3
            atk
            glib
            pango
            cairo
            gdk-pixbuf
            harfbuzz
            glib-networking
            librsvg
            xdotool
            alsa-lib
            linuxPackages.v4l2loopback
            v4l-utils
            libxkbcommon
            libx11
            libxtst
            libxi
            leptonica
            tesseract
          ];
      };
    });

    nixosModules = {
      default = self.nixosModules.lifelog-postgres;

      lifelog-postgres = {
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
            package = cfg.package;
            ensureDatabases = [cfg.database];
            ensureUsers = [
              {
                name = cfg.user;
                ensureDBOwnership = true;
              }
            ];
          };
        };
      };
    };
  };
}
