{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
    leptonica
    clang
    # Add any other dependencies your project needs
    tesseract
    glibc
  ];

  # Set environment variables needed for bindgen
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.clang.version}/include";

  NIX_CFLAGS_COMPILE = "-I${pkgs.glibc.dev}/include";
  # Tesseract needs these at runtime
  TESSDATA_PREFIX = "${pkgs.tesseract}/share/tessdata";
}
