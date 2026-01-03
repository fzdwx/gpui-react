#!/usr/bin/env just --justfile
export PATH := join(justfile_directory(), "node_modules", "bin") + ":" + env_var('PATH')

native:
    bun build:rust

demo:
    bun run demo
