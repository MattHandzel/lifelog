{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
    in {

      packages.${system}.lifelog= pkgs.rustPlatform.buildRustPackage {
        pname = "lifelog";
        version = "0.1.0";
        src = ./.;

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        buildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.alsa-lib
          pkgs.sqlite
        ];

        nativeBuildInputs = [
          pkgs.cmake
        ];

        meta = with pkgs.lib; {
          description = "A lifelogging project";
          license = licenses.mit;
          maintainers = [ maintainers.MattHandzel ];
        };
      };
      devShells.${system}.default = pkgs.mkShell {

        packages = [
          (pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          })
          pkgs.glibc
          pkgs.libxkbcommon
          pkgs.pkg-config
          pkgs.openssl
          pkgs.grim
          pkgs.alsa-lib
          pkgs.slurp
          pkgs.sqlite
        ];
      };
    };
}
