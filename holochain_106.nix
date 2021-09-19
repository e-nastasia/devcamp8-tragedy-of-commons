# Example: Custom Holochain And Binaries
# 
# The following `shell.nix` file can be used in your project's root folder and activated with `nix-shell`.
# It uses a custom revision and a custom set of binaries to be installed.

{ 
  holonixPath ?  builtins.fetchTarball { url = "https://github.com/holochain/holonix/archive/develop.tar.gz"; }
}:


let
  holonix = import (holonixPath) {
    include = {
        # making this explicit even though it's the default
        holochainBinaries = true;
    };

    holochainVersionId = "custom";

    holochainVersion = {
     rev = "b11908875a9f6a09e8939fbf6f45ff658e3d10a6";
     sha256 = "sha256:074jdpr0dmzp2wl7flal1rdcnhd86bbf9g8dmzq4wl616v3ibqzs";
     cargoSha256 = "sha256:0k1bbh7cklnhzkj06icbi7wamq6hl6q77d51k43qil3mrvddb7j0";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };

     lairKeystoreHashes = {
       sha256 = "sha256:0khg5w5fgdp1sg22vqyzsb2ri7znbxiwl7vr2zx6bwn744wy2cyv";
       cargoSha256 = "sha256:1lm8vrxh7fw7gcir9lq85frfd0rdcca9p7883nikjfbn21ac4sn4";
     };
    };
  };

  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    nodejs-16_x
  ];
}