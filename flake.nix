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
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [(import rust-overlay)];
    };
  in {
    packages.${system} = {
      lifelog-server = pkgs.rustPlatform.buildRustPackage {
        pname = "lifelog-server";
        version = "0.1.0";
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        buildInputs = with pkgs; [
          pkg-config
          openssl
          alsa-lib
          sqlite
          linuxPackages.v4l2loopback
          v4l-utils
          xorg.libX11
          xorg.libXtst
        ];

        cargoBuildFlags = ["--bin" "lifelog-server"];

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          alsa-lib
        ];

        meta = with pkgs.lib; {
          description = "A project to log various sources of data for your lifelog";
          license = licenses.mit;
          maintainers = [maintainers.MattHandzel];
        };
      };
      lifelog-logger = pkgs.rustPlatform.buildRustPackage {
        pname = "lifelog-logger";
        version = "0.1.0";
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        buildInputs = with pkgs; [
          pkg-config
          openssl
          alsa-lib
          sqlite
          linuxPackages.v4l2loopback
          v4l-utils
          xorg.libX11
          xorg.libXtst
        ];

        cargoBuildFlags = ["--bin" "lifelog-logger"];

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          alsa-lib
        ];

        meta = with pkgs.lib; {
          description = "A project to log various sources of data for your lifelog";
          license = licenses.mit;
          maintainers = [maintainers.MattHandzel];
        };
      };
    };
    devShells.${system}.default = pkgs.mkShell {
      PKG_CONFIG_PATH = "${pkgs.alsa-lib}/lib/pkgconfig";

      packages = with pkgs; [
        (rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        })
        openssl
        glibc
        linuxPackages.v4l2loopback
        v4l-utils
        libxkbcommon
        pkg-config
        alsa-lib
        grim
        xorg.libX11
        slurp
        xorg.libXtst
        sqlite
      ];
    };
  };
}
