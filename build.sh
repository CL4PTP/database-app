#!/bin/bash

wasm-pack build

wasm2js --pedantic ./pkg/dbapp_bg.wasm --output ./pkg/dbapp_bg.js

uglifyjs --compress --mangle --output ./pkg/dbapp_bg.min.js ./pkg/dbapp_bg.js
