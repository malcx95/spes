let pkgs = import <nixpkgs> { };
in pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    cargo-watch

    libGL
    xorg.libX11
    xorg.libXi

    alsa-lib
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
      with pkgs;
      pkgs.lib.makeLibraryPath [ libGL xorg.libX11 xorg.libXi ]
    }"
  '';
}
