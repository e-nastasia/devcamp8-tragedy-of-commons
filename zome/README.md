# DNA: Tragedy of commons

This folder contains entire backend for the Tragedy of commons game implementation.
Actual code for the zome is in the `zomes/tragedy_of_commons`.

(e-nastasia) It is recommended to run VSCode from this directory (1) and outside of the nix-shell (2):
  (1): root level directory doesn't have the `Cargo.toml` for rust-analyzer to read, 
  (2): nix-shell would mess up VSCode paths to Rust binaries (or maybe I just don't know how to set it up!)

**Attention**: note that all commands below assume that you're running them in nix-shell that has been started from the root of this repository: it has `default.nix` which is used to set up nix-shell properly.

## Building

For your convenience, there's a bash script that already contains all the necessary commands:

```bash
./run_build.sh
```

This should create a `workdir/happ/tragedy_of_commons.happ` file.

## Testing

After running the builds with the bash script above you can use another one for testing:

```bash
./run_tests.sh
```

## Running

After having built the DNA:

```bash
hc s generate workdir/happ/tragedy_of_commons.happ --run=8888
```

Now `holochain` will be listening at port `8888`;

## Running in the sandbox

Alternatively you can use sandbox setup (TODO: explain the difference between execution envs for `hc s --run` and sandbox) that is already configured to allow for two conductors (most useful for UI testing).
The script to run it is `workdir/happ/run_sandbox.sh` and you would need to run it from this directory:

```bash
cd workdir/happ/
./run_sandbox.sh
```

That would create two conductors: one on 8000 and another on 8001 ports.


## Implementation

### Flow

What is already implemented:
- first player creates a game code: `create_game_code_anchor`
- they share this code outside of the app with everyone else who wants to play
- all players join the game with this code: `join_game_with_code`
- first player starts the game: `start_game_session_with_code`
- players make moves for the current round: `make_new_move`
- their UIs try to close the round after making a move, in case everybody else made the moves: `try_to_close_round`
