# Zome Developer Setup

This folder has an example DNA for the `tragedy_of_commons` zome. The actual code for the zome is in `zomes/tragedy_of_commons`.

To change the code, you can work either opening VSCode inside the root folder of the repo or in this folder, you should have rust intellisense either way.

All the instructions here assume you are running them inside the nix-shell at the root of the repository. For more info, see the [developer setup](/dev-setup.md).

## Building

```bash
CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown
hc dna pack workdir/dna
hc app pack workdir/happ
```

This should create a `workdir/happ/sample.happ` file.

## Testing

After having built the DNA:

```bash
cd test
npm install
npm test
```

## Running

After having built the DNA:

```bash
hc s generate workdir/happ/sample.happ --run=8888
```

Now `holochain` will be listening at port `8888`;

----------------------------

## Implementation

### Flow

What is already implemented:
- first player creates a game code: `create_game_code_anchor`
- they share this code outside of the app with everyone else who wants to play
- all players join the game with this code: `join_game_with_code`
- first player starts the game: `start_game_session_with_code`
- players make moves for the current round: `make_new_move`
- their UIs try to close the round after making a move, in case everybody else made the moves: `try_to_close_round`
