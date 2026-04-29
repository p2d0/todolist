{
  description = "pomotasker";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];

      overlay = final: prev: {
        pomotasker = final.rustPlatform.buildRustPackage {
          name = "pomotasker";
          version = "0.1.0";

          src = ./.;

          nativeBuildInputs = [ final.pkg-config final.wrapGAppsHook4 ];

          buildInputs = [
            final.gtk4
            final.cairo
            final.glib
            final.pango
            final.gdk-pixbuf
            final.graphene
            final.sqlite
          ];

          cargoLock.lockFile = ./Cargo.lock;

          postInstall = ''
            mkdir -p $out/share/applications
            cp ${./share/applications/com.pomotasker.app.desktop} $out/share/applications/
          '';

          NIX_LDFLAGS = "-rpath ${
            final.lib.makeLibraryPath [
              final.gtk4
              final.cairo
              final.glib
              final.pango
              final.gdk-pixbuf
              final.graphene
              final.sqlite
            ]
          }";

          meta = with final.lib; {
            description = "Pomodoro habit tracker with GTK4";
            homepage = "https://github.com/patriot720/pomotasker";
            license = licenses.gpl2Only;
            platforms = platforms.linux;
            mainProgram = "pomotasker";
          };
        };
      };
    in
    {
      overlays.default = overlay;
    } // (flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in
      {
        packages = {
          default = pkgs.pomotasker;
          pomotasker = pkgs.pomotasker;
        };
      }
    ));
}
