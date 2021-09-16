let 
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/014d28000c8ed021eb84000edfe260c22e90af8c.tar.gz";
    sha256 = "sha256:0hl5xxxjg2a6ymr44rf5dfvsb0c33dq4s6vibva6yb76yvl6gwfi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";
    
    holochainVersion = { 
     rev = "792c707e8abeb9566de6ddac04f699f208b923ff";  
     sha256 = "sha256:0v89ginakm4zj9sf4n24hzr7pn4mdq362qwccbphw6dg60jl5v7d";
     cargoSha256 = "sha256:1i8sgf1pamjzlx9p62dm81b795z8gvgcd51w42ivxwfp8jq95qrx";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main