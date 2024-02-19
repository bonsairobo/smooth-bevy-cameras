{
  description = "NixOS environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
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
        ###
        ## Packages
        ###

        xLibs = with pkgs; [
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        dynLibs = with pkgs;
          [
            alsa-lib
            stdenv.cc.cc.lib
            udev
            vulkan-loader
          ]
          ++ xLibs;

        buildInputs = with pkgs;
          [
            clang
            llvmPackages_16.bintools
            mold
            pkg-config
            rustup
            yq-go
          ]
          ++ dynLibs;

        ###
        ## Rust Toolchain Setup
        ###

        shellHook = ''
          export RUSTC_VERSION=$(yq ".toolchain.channel" rust-toolchain.toml)
          export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
          export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          rustup target add wasm32-unknown-unknown
          rustup component add rust-analyzer
        '';

        ###
        ## Rust Bindgen Setup
        ###

        # So bindgen can find libclang.so
        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_16.libclang.lib];
        # Add headers to bindgen search path
        BINDGEN_EXTRA_CLANG_ARGS = [
          ''-I"${pkgs.llvmPackages_16.libclang.lib}/lib/clang/${pkgs.llvmPackages_16.libclang.version}/include"''
        ];

        ###
        ## Linking with System libraries
        ###

        # Add precompiled library to rustc search path
        RUSTFLAGS = builtins.map (a: ''-L ${a}/lib'') dynLibs;

        # Some libraries must be in the dynamic linker path.
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.libxkbcommon pkgs.vulkan-loader];
      };
  };
}
