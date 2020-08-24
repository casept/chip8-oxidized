let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  niv = import sources.niv { };
  nixpkgs = import sources.nixpkgs { };
in nixpkgs.mkShell { buildInputs = [ rust nixpkgs.SDL2 nixpkgs.wasm-pack nixpkgs.wasm-bindgen-cli nixpkgs.nodejs niv.niv ]; }
