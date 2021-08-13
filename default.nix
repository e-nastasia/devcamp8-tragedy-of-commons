let 
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/014d28000c8ed021eb84000edfe260c22e90af8c.tar.gz";
    sha256 = "0hl5xxxjg2a6ymr44rf5dfvsb0c33dq4s6vibva6yb76yvl6gwfi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";
    
    holochainVersion = { 
     rev = "3a47f9798c6175997d27d450a7c4a0b92d17d4da";
     sha256 = "07y9bl9q0nzwrb470walxsa8d9254mcns75hai83v1qbcjmfli4j";
     cargoSha256 = "0sbr6ag5dyv7gdchpp5zjwzbsr1ggvxnmik21wm71gjd8hhwcdy6";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main