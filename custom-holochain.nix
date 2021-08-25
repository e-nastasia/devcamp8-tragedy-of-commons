#custom nix-shell for when we want to test a new holochain version
let 
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/014d28000c8ed021eb84000edfe260c22e90af8c.tar.gz";
    sha256 = "sha256:0hl5xxxjg2a6ymr44rf5dfvsb0c33dq4s6vibva6yb76yvl6gwfi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";
    
    holochainVersion = { 
     rev = "f3d17d993ad8d988402cc01d73a0095484efbabb";  
     sha256 = "sha256:1z0y1bl1j2cfv4cgr4k7y0pxnkbiv5c0xv89y8dqnr32vli3bld7";
     cargoSha256 = "sha256:1rf8vg832qyymw0a4x247g0iikk6kswkllfrd5fqdr0qgf9prc31";
     bins = {
       holochain = "holochain";
       hc = "hc";
       kitsune-p2p-proxy = "kitsune_p2p/proxy";
      };

      lairKeystoreHashes = {
        sha256 = "1jiz9y1d4ybh33h1ly24s7knsqyqjagsn1gzqbj1ngl22y5v3aqh";
        cargoSha256 = "0000000000000000000000000000000000000000000000000000";
      };
    };
  };
in holonix.main