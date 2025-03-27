#!/usr/bin/env bash

bun build --compile --target=bun-linux-x64 ./index.ts --outfile target/cover-loader
bun build --compile --target=bun-windows-x64-baseline ./index.ts --outfile target/cover-loader

