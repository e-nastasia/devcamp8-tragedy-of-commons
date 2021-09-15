# Developer Setup

## Requirements

- Having [`nix-shell` installed](https://developer.holochain.org/docs/install/).

## nix-shell

Once you are in the nix-shell you can run your cargo tests in this project:

RUSTFLAGS='-A warnings' cargo test --features "mock"

// Run all the commands specified in the documentation inside the nix-shell provided by the `*.nix` files at the root of this repository.

// To enter the nix-shell at the root of this repository, simply run `nix-shell` in it.

## Structure

This respository is structured in the following way:

- `ui/`: UI library.
- `zome/`: example DNA with the `todo_rename` code.
- Top level `Cargo.toml` is a virtual package necessary for other DNAs to include this zome by pointing to this git repository.
- Top level `*.nix` files define the nix environment needed to develop with this repository.

Read the [UI developer setup](/ui/README.md) and the [Zome developer setup](/zome/README.md).


# Run game in online mode

after compiling the same source code as the other player
you run
> hc sandbox generate --run=8000 --app-id=tragedy_of_commons --num-sandboxes=1 network --bootstrap https://bootstrap-staging.holo.host quic

This generates a sandbox. Look in the output for a file path and name like (name will differ):
/tmp/tmp.LR0icGixJK/aBI6KPWClY8jF6uYh7m35  

And open the conductor-config.yml file and replace the transport pool section with this:

  transport_pool:
    - type: proxy
      sub_transport:
        type: quic
        bind_to: ~
        override_host: ~
        override_port: ~
      proxy_config:
        type: remote_proxy_client
        proxy_url: "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--"
  
Check with :
> hc list

It should only show 1 /tmp/tmp.xyz**** file
If you have many then run
> hc clean
And start going through these steps again

then run 
hc sandbox run 0

this runs the sandbox, but with the changed transport settings

run the UI, generate a game code and attempt to play online