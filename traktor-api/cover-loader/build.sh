#!/usr/bin/env bash

bun build --compile --target=bun-linux-x64 ./index.ts --outfile target/cover-loader-linux-x64
bun build --compile --target=bun-linux-aarch64 ./index.ts --outfile target/cover-loader-linux-aarch64
bun build --compile --target=bun-windows-x64-baseline ./index.ts --outfile target/cover-loader-windows-x64

