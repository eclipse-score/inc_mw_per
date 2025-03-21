{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  packages = [
    # stddef.h
    pkgs.llvmPackages.clangUseLLVM
  ];

  env = {
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    BINDGEN_EXTRA_CLANG_ARGS = "-I ${pkgs.linuxHeaders}/include -I ${pkgs.glibc.dev}/include";
  };
}
