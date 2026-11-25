
{
  description = "Minimal e-paper framebuffer viewer";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };

          python = pkgs.python3.withPackages (ps: [
            ps.tkinter
          ]);
        in
        {
          default = pkgs.mkShell {
            packages = [
              python
              pkgs.tcl
              pkgs.tk
            ];

            shellHook = ''
              export TCL_LIBRARY="${pkgs.tcl}/lib"
              export TK_LIBRARY="${pkgs.tk}/lib"
            '';
          };
        });
    };
}

