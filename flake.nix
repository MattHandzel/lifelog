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
          pkgs.alsaLib
          pkgs.slurp
        ];
      };
    };
}
