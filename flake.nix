{
  description = "NixOS environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShell.${system} = with pkgs;
      mkShell rec {
        # Rust binaries need to be able to link and load some system libraries.
        buildInputs = [
          libxkbcommon
          pkg-config
          udev
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
      };
  };
}
