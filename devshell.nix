{ pkgs }:

with pkgs;

# Configure your development environment.
devshell.mkShell {
  name = "pomotasker";
  motd = ''
Entered PomoTasker development environment.
'';
  env = [
    {
      name = "PKG_CONFIG_PATH";
      value = "${lib.concatStringsSep ":" [
        "${glib.dev}/lib/pkgconfig"
        "${pango.dev}/lib/pkgconfig"
        "${gdk-pixbuf.dev}/lib/pkgconfig"
        "${cairo.dev}/lib/pkgconfig"
        "${graphene.dev}/lib/pkgconfig"
        "${gtk4.dev}/lib/pkgconfig"
        "${sqlite.dev}/lib/pkgconfig"
        "${harfbuzz.dev}/lib/pkgconfig"
        "${vulkan-loader.dev}/lib/pkgconfig"
      ]}";
    }
    {
      name = "LIBCLANG_PATH";
      value = "${libclang.lib}/lib";
    }
    {
      name = "LD_LIBRARY_PATH";
      value = "${wayland}/lib:${libxkbcommon}/lib";
    }
  ];
  packages = [
    libclang.lib
    wayland
    glib
    pango
    gdk-pixbuf
    cairo
    graphene
    gtk4
    sqlite
    harfbuzz
    vulkan-loader
    pkg-config
    rustc
    cargo
    rustfmt
    clippy
    rust-analyzer
    gdb
  ];
}
