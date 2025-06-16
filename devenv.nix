{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = ["rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "llvm-tools-preview"];
  };
}
