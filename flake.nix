{
  description = "x86-64-emulator";

  inputs = {
    rust.url = "github:nathenjacobe/rust-flake";
    nixpkgs.follows = "rust/nixpkgs";
  };

  outputs = { nixpkgs, rust, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
      };

      # runtime libraries needed by the egui/eframe (glow) frontend: X11,
      # Wayland and OpenGL; exposed on LD_LIBRARY_PATH so the binaries built in
      # this shell can find them at runtime on NixOS
      guiLibs = with pkgs; [
        libxkbcommon
        wayland
        wayland-protocols
        libx11
        libxcursor
        libxrandr
        libxi
        libxcb
        libGL
        mesa
        fontconfig
        freetype
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          rust.packages.${system}.rustToolchain
          pkgs.pkg-config
        ] ++ guiLibs;

        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath guiLibs}:''${LD_LIBRARY_PATH:-}"
        '';
      };
    };
}
