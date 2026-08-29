{
  description = "Query battery status from supported Razer devices";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/26.05";

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          razer = pkgs.callPackage ./default.nix { };
          default = pkgs.callPackage ./default.nix { };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.razer ];

            packages = with pkgs; [
              rust-analyzer
              rustfmt
              clippy
              taplo
              usbutils
            ];
          };
        }
      );
    };
}
