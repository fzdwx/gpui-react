#!/usr/bin/env just --justfile
export PATH := join(justfile_directory(), "node_modules", "bin") + ":" + env_var('PATH')

native:
    bun build:rust

demo:
    bun run demo

release:
    bun run build
    git add .
    bun run release
    npm publish  --registry https://registry.npmjs.org