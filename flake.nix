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

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "lifelog";
        version = "0.1.0";
        src = ./.;
        cargoSha256 = "sha256-0000000000000000000000000000000000000000000000000000"; 

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        buildInputs = [
          pkgs.pkg-config
          pkgs.openssl
          pkgs.alsa-lib
        ];

        nativeBuildInputs = [
          pkgs.cmake
        ];

        meta = with pkgs.lib; {
          description = "My Rust Project";
          license = licenses.mit;
          maintainers = [ maintainers.yourname ];
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
        ];
      };
    };
}
