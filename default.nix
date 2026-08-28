{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage {
  pname = "razer";
  version = "0.1.0";

  src = pkgs.lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    systemd
  ];

  meta = {
    description = "Query battery status from supported Razer devices";
    mainProgram = "razer";
    platforms = pkgs.lib.platforms.linux;
  };
}
