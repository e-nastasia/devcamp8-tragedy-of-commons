# Build UI
cd ui/svelte-ui
npm run build
rm dist.zip
cd public
zip -r ../dist.zip *
cd ../../../

# Build happ
cd zome
sh run_build.sh
hc web-app pack workdir/webhapp

