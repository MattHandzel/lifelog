let
  rustOverlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> {overlays = [rustOverlay];};
  rustVersion = "latest"; # or specify a version like "nightly"
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = ["rust-src" "rust-analyzer"];
  };
in
  pkgs.mkShell {
    buildInputs = [rust];
  }
