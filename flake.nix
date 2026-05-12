{
  description = "LnMai dev shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: builtins.listToAttrs (map (system: { name = system; value = f system; }) systems);
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          runtimeLibs = with pkgs; [
            libx11
            libxcursor
            libxrandr
            libxi
            libxext
            libxinerama
            libxcb
            libxkbcommon
            libglvnd
          ];
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              assimp
              elan
              rustc
              cargo
              pkg-config
              openssl
              clang
              cmake
              git
              libx11
              libxcursor
              libxrandr
              libxi
              libxext
              libxinerama
              libxcb
              libxkbcommon
              libglvnd
            ];

            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
              echo "LnMai dev shell ready"
            '';
          };
        });
    };
}
