with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "sdkgen";

  buildInputs = [
    stdenv
    pkg-config
  ];
}
