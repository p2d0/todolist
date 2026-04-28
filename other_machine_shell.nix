{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  name = "pomotasker-run";
  packages = [
    pkgs.gtk4
    pkgs.cairo
    pkgs.glib
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.graphene
    pkgs.sqlite
  ];
  shellHook = ''
    export LD_LIBRARY_PATH="${
      pkgs.lib.makeLibraryPath [
        pkgs.gtk4
        pkgs.cairo
        pkgs.glib
        pkgs.pango
        pkgs.gdk-pixbuf
        pkgs.graphene
        pkgs.sqlite
      ]
    }"
    echo "Run: ./target/release/pomotasker"
  '';
}
