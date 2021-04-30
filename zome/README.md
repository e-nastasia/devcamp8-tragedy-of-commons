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

- player decides to create a new game and invites other people to it
    - at this point, the invite zome from Manuel (Guillem has more details) will take care of:
        - creating invites
        - managing their state (pending, accepted, declined)
    - game session is only created once everyone answered their invites, only for accepted players
- once the session is created, players are notified that they can make their moves
- every player makes a single move for the first round
- first round is created (==closed) once all accepted players make their moves
- new round starts immediately after that and all players can make another move
- this happens until 