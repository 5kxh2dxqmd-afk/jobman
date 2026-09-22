{
  description = "Jobman build environment (Kindle cross toolchain, Rust + zig, autotools)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      # koxtoolchain only publishes x86_64 Linux host binaries
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      # Prebuilt koxtoolchain for firmware >= 5.16.3 (hard float). nixpkgs' own
      # cross compilers can't be used: binaries must link against the device's
      # glibc 2.20, which this toolchain's sysroot provides.
      koxtoolchain = pkgs.stdenv.mkDerivation {
        pname = "koxtoolchain-kindlehf";
        version = "2026.08";

        src = pkgs.fetchurl {
          url = "https://github.com/koreader/koxtoolchain/releases/download/2026.08/kindlehf.tar.zst";
          hash = "sha256-jMffvXGr14+elH1rLiBnAoikQC7cewcXa8p5H36vh9A=";
        };
        sourceRoot = "x-tools/arm-kindlehf-linux-gnueabihf";

        nativeBuildInputs = [ pkgs.zstd pkgs.autoPatchelfHook ];
        buildInputs = [ pkgs.stdenv.cc.cc.lib ];

        dontConfigure = true;
        dontBuild = true;
        # The sysroot is full of ARM binaries and scripts meant for the device;
        # only the x86_64 host tools get patched.
        dontStrip = true;
        dontPatchELF = true;
        dontAutoPatchelf = true;
        dontPatchShebangs = true;

        installPhase = ''
          cp -a . $out
        '';

        postFixup = ''
          autoPatchelf $out/bin $out/libexec $out/lib
          patchShebangs $out/bin
        '';
      };

      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "armv7-unknown-linux-musleabihf" ];
      };

      # Libraries slint's default desktop backend dlopens for `./build.sh local`
      desktopLibs = with pkgs; [
        fontconfig
        libGL
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
      ];
    in
    {
      packages.${system}.koxtoolchain = koxtoolchain;

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          koxtoolchain
          rust
          pkgs.cargo-zigbuild
          pkgs.zig
          pkgs.autoconf
          pkgs.automake
          pkgs.gnumake
          pkgs.patch
          pkgs.git
        ];

        KINDLE_TC = "${koxtoolchain}";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath desktopLibs;
      };
    };
}
