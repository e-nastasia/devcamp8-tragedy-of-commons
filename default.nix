let 
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/f8bf0836b83ec6515e9da66a29302f18d298bbfa.tar.gz";
    sha256 = "1y3ligglx22yigjlhv0sman24arz2dqilrprp1nlnz5r02xsl13d";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";
    
    holochainVersion = { 
     rev = "834d01d7f1f8af44c9db64f116a9f5b56856a4d1";
     sha256 = "1d0x42g2q20idayr0hg4m7fqvjjz0sbmkgfdhcshzwwbmy69rjc9";
     cargoSha256 = "08lklvsp0dywwm8smv5y0zf28pq74ssmx42ij1nyz2xwq124szkj";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
  };
in holonix.main
