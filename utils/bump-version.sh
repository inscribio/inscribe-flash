#!/usr/bin/env bash

set -euo pipefail

json_version=$(jq '.version' package.json -r)
toml_version=$(grep -E '^version = "[0-9.]+"' src-tauri/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

echo "Old version:"
echo "  package.json: $json_version"
echo "  Cargo.toml:   $toml_version"

if (( $# < 1 )); then
    echo "Usage: $(basename "$0") [ NEW_VERSION ]"
    echo "Update application version"
    exit 0
fi

new_version="$1"

echo "New version: $new_version"
read -rep "Proceed? [y/N] " choice
[[ "$choice" == [Yy]* ]] || exit 1


# update package.json
jq ".version = \"$new_version\"" package.json | sponge package.json

# update Cargo.toml
sed -i "s!^version = \"[0-9.]\+\"!version = \"$new_version\"!" src-tauri/Cargo.toml

# update version in package-lock.json
npm install .

# update Cargo.lock
just build
