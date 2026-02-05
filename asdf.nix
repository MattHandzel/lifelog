# {pkgs ? import <nixpkgs> {}}:
# with pkgs;
#   mkShell {
#     name = "lifelog-shell";
#
#     buildInputs = [
#       glib
#       clang
#       llvmPackages.libcxxClang
#       libclang.lib
#       pkg-config
#     ];
#
#     BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.versions.major (lib.getVersion clang)}/include";
#     LIBCLANG_PATH = "${libclang.lib}/lib";
#
#     shellHook = ''
#       # Mirror what gcc-wrapper would do:
#       export CFLAGS="$(< ${stdenv.cc}/nix-support/cc-cflags)"
#       export CXXFLAGS="$(< ${stdenv.cc}/nix-support/libcxx-cxxflags)"
#     '';
#   }
