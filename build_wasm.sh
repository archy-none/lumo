cd ./wasm
wasm-pack build --target nodejs
mv pkg/* ../docs/wasm/node/
