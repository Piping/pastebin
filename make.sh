#!/bin/bash

SCRIPTDIR=$(dirname "$0")

bs() {
    sudo apt install entr lld
    mkdir -p "$SCRIPTDIR/upload"
}

dev() {
    find . -name '*.rs' -or -name '*.toml' | entr -c -r cargo run --release
}

"$@"
