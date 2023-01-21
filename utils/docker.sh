#!/usr/bin/env bash

set -euo pipefail

HERE="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
cd "$HERE/.."

out="$PWD/docker-bundle"
container_out="/flasher/src-tauri/target/release/bundle"

docker build -t flasher:v1 .
docker run -it --rm -v "$out:$container_out" flasher:v1
