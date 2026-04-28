{
  nixConfig = {
    extra-substituters = [
      "https://upgradegamma.cachix.org"
    ];
    extra-trusted-public-keys = [
      "upgradegamma.cachix.org-1:iIifduPUNZ9OrRYgaEcKTeRQxbqr2/FbiF1bboND05A="
    ];
  };

  description = "pomotasker";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    devshell.url = "github:numtide/devshell";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, naersk, devshell, flake-utils }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "armv7l-linux" ];

      overlay = final: prev: {
        pomotasker = final.callPackage ({ lib, stdenv, rustPlatform, pkg-config, gtk4, cairo, glib, pango, gdk-pixbuf, wrapGAppsHook4, sqlite}: 
          let
            toolchain = fenix.packages.${stdenv.hostPlatform.system}.minimal.toolchain;
            naersk-lib = naersk.lib.${stdenv.hostPlatform.system}.override {
              cargo = toolchain;
              rustc = toolchain;
            };
            runtimeDeps = [
              gtk4
              cairo
              glib
              pango
              gdk-pixbuf
              sqlite
            ];
          in naersk-lib.buildPackage {
            src = ./.;
            nativeBuildInputs = [ pkg-config wrapGAppsHook4 ];
            buildInputs = runtimeDeps;

            # Robust fix: Force the linker to include ALL runtime dependencies in the RPATH
            NIX_LDFLAGS = "-rpath ${lib.makeLibraryPath runtimeDeps}";

            meta = with lib; {
              description = "Pomodoro habit tracker with GTK4";
              homepage = "https://github.com/patriot720/pomotasker";
              license = licenses.gpl2Only;
              platforms = platforms.linux;
            };
          }) {};
      };

      nixosModule = { config, lib, pkgs, ... }:
        let
          cfg = config.services.pomotasker;
        in {
          options.services.pomotasker = {
            enable = lib.mkEnableOption "PomoTasker pomodoro habit tracker";
            package = lib.mkOption {
              type = lib.types.package;
              default = pkgs.pomotasker;
              description = "The pomotasker package to use.";
            };
          };

          config = lib.mkIf cfg.enable {
            nixpkgs.overlays = [ self.overlays.default ];

            systemd.user.services.pomotasker = {
              description = "PomoTasker pomodoro habit tracker";
              wantedBy = [ "graphical-session.target" ];
              partOf = [ "graphical-session.target" ];
              after = [ "graphical-session.target" ];
              serviceConfig = {
                ExecStart = "${cfg.package}/bin/pomotasker";
                Restart = "always";
                RestartSec = 3;
              };
            };
          };
        };
    in
    {
      inherit overlay;
      overlays.default = overlay;
      nixosModules.default = nixosModule;
      homeManagerModules.default = nixosModule;
    } // (flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default devshell.overlays.default ];
        };

        buildFor = ({ target, cross }:
          let
            toolchain = with fenix.packages.${system}; combine [
              minimal.cargo
              minimal.rustc
              targets.${target}.latest.rust-std
            ];
            packages = nixpkgs.legacyPackages.${system};
          in
          (naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
            pkgs = packages.pkgsCross.${cross};
          }).buildPackage {
            src = ./.;
            RUSTFLAGS = [
              "-C"
              "target-feature=+crt-static"
              "-L ${packages.pkgsCross.${cross}.glibc.static}/lib"
            ];
            autoCrateSpecificOverrides = true;
            CARGO_BUILD_TARGET = target;
            TARGET_CC = "${packages.pkgsCross.${cross}.stdenv.cc}/bin/${packages.pkgsCross.${cross}.stdenv.cc.targetPrefix}cc";
          });
      in
      {
        packages = {
          default = pkgs.pomotasker;
          pomotasker = pkgs.pomotasker;
          armv7 = buildFor { target = "armv7-unknown-linux-musleabihf"; cross = "armv7l-hf-multiplatform"; };
          armv7-gnu = buildFor { target = "armv7-unknown-linux-gnueabihf"; cross = "armv7l-hf-multiplatform"; };
          aarch64 = buildFor { target = "aarch64-unknown-linux-musl"; cross = "aarch64-multiplatform-musl"; };
        };

        devShell = import ./devshell.nix { inherit pkgs; };
      }
    ));
}
