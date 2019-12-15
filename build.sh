#!/bin/sh
set -e -o pipefail
wasm-pack build --target web --out-dir dist
cp src/index.html src/style.css dist
