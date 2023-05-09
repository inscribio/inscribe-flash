#!/usr/bin/env bash

set -euo pipefail

# Poor-but-enough substitution of realpath when its not available on the system
command -v realpath >/dev/null 2>&1 || \
    realpath() {
        [[ $1 = /* ]] && echo "$1" || echo "$PWD/${1#./}"
    }

HERE="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

die() {
    echo "$@" 1>&2
    exit 1
}

pkg_version() {
    if command -v jq &> /dev/null; then
        jq '.version' ./package.json -r
    else
        pushd "$HERE/.." &> /dev/null
        node -p 'require("./package.json").version'
        popd &> /dev/null
    fi
}

pkg_arch() {
    if [[ "$rust_target" = 'x86_64-apple-darwin' ]]; then
        echo x64
    elif [[ "$rust_target" = 'aarch64-apple-darwin' ]]; then
        echo aarch64
    else
        die "Unknown target: $rust_target"
    fi
}

if (( $# < 1 )); then
    echo "Usage: $(basename $0) RUST_TARGET"
    echo "Create ZIP with MacOs .app adding correct version and architecture to the name."
    exit 1
fi

rust_target="$1"
zip_name="inscribe-flash_$(pkg_version)_$(pkg_arch).app.zip"

cd "$HERE/../src-tauri/target/$rust_target/release/bundle/macos"

zip -9 -r "$zip_name" ./*.app
