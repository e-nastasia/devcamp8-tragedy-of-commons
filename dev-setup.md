# Developer Setup

## Requirements

- Having [`nix-shell` installed](https://developer.holochain.org/docs/install/).

## nix-shell

Any builds for this project need to happen inside the nix-shell which you need to start by running `nix-shell` command in the root folder of this repo (right now the same level as this doc). Running `nix-shell` in the repo root would rely on `default.nix` for the specific Holochain version and all of it's dependencies.

## Structure

This respository is structured in the following way:

- `ui/`: currently we have multiple UI attempts. The latest one is in `ui/svelte-ui`
- `zome/`: backend folder. See more docs about the backend there, run VSCode from that directory.
- `default.nix`: configuration for nix-shell and Holochain versions to be used when building this project

Read the [UI developer setup](/ui/svelte-ui/README.md) and the [Zome developer setup](/zome/README.md).


# Run game in online mode

cd zome/workdir/happ/
run ./run_sandbox_remove_network.sh
run the UI, generate a game code and attempt to play online
