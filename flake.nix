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
          pname = "pomotasker";
          version = "0.1.0";

          src = ./.;

          nativeBuildInputs = [ 
            final.pkg-config 
            final.wrapGAppsHook4 
            final.copyDesktopItems # This helps automate desktop file installation
          ];

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

          # Define the desktop item here
          desktopItems = [
            (final.makeDesktopItem {
              name = "com.pomotasker.app";
              exec = "pomotasker";
              icon = "com.pomotasker.app";
              desktopName = "PomoTasker"; # Use desktopName for the display name
              comment = "Pomodoro habit tracker";
              terminal = false;
              categories = [ "Utility" ];
            })
          ];

          postInstall = ''
      # Install the icon (Assuming you have an icon file in your source)
      # Replace 'assets/icon.png' with the actual path to your icon in your repo
      install -D assets/icon.png $out/share/icons/hicolor/128x128/apps/com.pomotasker.app.png
      install -D assets/icon.svg $out/share/icons/hicolor/scalable/apps/com.pomotasker.app.svg
    '';

          # Optional: Only use this if you get "library not found" errors at runtime.
          # wrapGAppsHook4 usually handles this automatically.
          # NIX_LDFLAGS = ... (removed for brevity, keep if you specifically need it)

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
