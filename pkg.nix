{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage {
  name = "pomotasker";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [ pkgs.pkg-config pkgs.wrapGAppsHook4 ];

  buildInputs = [
    pkgs.gtk4
    pkgs.cairo
    pkgs.glib
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.graphene
    pkgs.sqlite
  ];

  cargoLock.lockFile = ./Cargo.lock;

  NIX_LDFLAGS = "-rpath ${
      pkgs.lib.makeLibraryPath [
        pkgs.gtk4
        pkgs.cairo
        pkgs.glib
        pkgs.pango
        pkgs.gdk-pixbuf
        pkgs.graphene
        pkgs.sqlite
      ]
    }";

  meta = with pkgs.lib; {
    description = "Pomodoro habit tracker for Hyprland with GTK4 overlay";
    homepage = "https://github.com/patriot720/pomotasker";
    license = licenses.gpl2Only;
    platforms = platforms.linux;
    mainProgram = "pomotasker";
  };
}
