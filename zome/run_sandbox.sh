cd workdir/happ
hc sandbox clean
hc sandbox generate --run=8000,8001 --app-id=tragedy_of_commons --num-sandboxes=2 network --bootstrap https://bootstrap-staging.holo.host quic