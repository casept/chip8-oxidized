{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  date = "2020-08-22";
  targets = [ "wasm32-unknown-unknown" ];
  extensions = [ "rust-src" "rust-analysis" "rustfmt-preview" ];
  rustChannelOfTargetsAndExtensions = channel: date: targets: extensions:
    (pkgs.rustChannelOf { inherit channel date; }).rust.override {
      inherit targets extensions;
    };
  chan = rustChannelOfTargetsAndExtensions channel date targets extensions;
in chan
