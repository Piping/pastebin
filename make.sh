#!/bin/bash

SCRIPTDIR=$(dirname "$0")

bs() {
    sudo apt install entr lld
    pip3 install livereload
    mkdir -p "$SCRIPTDIR/upload"
}

livereload() {
    python3 - <<EOF
from livereload import Server
import time
import subprocess
def reload_when_ready():
    command=['cargo', 'build']
    return_code = subprocess.call(command)
    while return_code!= 0:
        time.sleep(3)
        return_code = subprocess.call(command)
server = Server()
server.watch('src/*', reload_when_ready)
server.serve(open_url="127.0.0.3:8000", open_url_delay=2, host='127.0.0.3', port='35729')
EOF
}

dev() {
    livereload &
    find . -name '*.rs' -or -name '*.toml' | entr -c -r env ROCKET_ENV=stage cargo run
}

deploy() {
    cargo build --release
}

reload() {
    pkill pastebin
}

"$@"
