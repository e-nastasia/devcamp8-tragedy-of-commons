{ 
  # https://github.com/holochain/holonix/commit/55a5eef58979fb6bc476d8c3e0c028cdeb1b5421
  # holochain 0.0.103
  holonixPath ?  builtins.fetchTarball { url = "https://github.com/holochain/holonix/archive/55a5eef58979fb6bc476d8c3e0c028cdeb1b5421.tar.gz"; }
}:

let
  holonix = import (holonixPath) {
    include = {
        # making this explicit even though it's the default
        holochainBinaries = true;
    };
    #just using default holochain version in this holonix setup
    holochainVersionId = "main";
  };

in holonix.main