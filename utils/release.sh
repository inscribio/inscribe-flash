#!/usr/bin/env bash

die() {
    echo "$@" 1>&2
    exit 1
}

HERE="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
root_dir="$HERE/.."
cd "$root_dir" || exit 1

out_dir="${1:-$root_dir/release}"
bundle_dir="$root_dir/src-tauri/target/release/bundle"

# Retrieve package version
if command -v jq &> /dev/null; then
    version=$(jq '.version' ./package.json -r)
else
    version=$(node -p 'require("./package.json").version')
fi

# Source paths
app_name='inscribe-flash'

win_msi="${bundle_dir}/msi/${app_name}_${version}_x64_en-US.msi"
mac_app="${bundle_dir}/macos/${app_name}.app"
mac_dmg="${bundle_dir}/dmg/${app_name}_${version}_x64.dmg"
linux_appimg="${bundle_dir}/appimage/${app_name}_${version}_amd64.AppImage"
linux_deb="${bundle_dir}/deb/${app_name}_${version}_amd64.deb"

# Generate output
mkdir -p "$out_dir"

generate() {
    name="$1"
    src="$2"
    flag="$3"
    if [[ -e "$src" ]]; then
        echo "Preparing $name ..." 1>&2
        out="${out_dir}/${app_name}_${version}_${name}"
        if [[ $flag = '--zip' ]]; then
            out="${out}.zip"

            # ensure archive contains only the needed file/directory
            dir="$(dirname "$src")"
            base="$(basename "$src")"
            pushd "$dir" &> /dev/null || die "Could not change directory"
            zip -9 -r "$out" "$base"
            popd &> /dev/null || die "Could not change directory"

            # du -sh "$out" "$src"
        elif ! [[ $flag = '' ]]; then
            echo "[ERROR] Unknown flag $flag" 1>&2
            exit 1
        else
            cp "$src" "$out"
        fi
    else
        echo "[WARNING] Missing $name sources" 1>&2
    fi
}

generate "Win" "$win_msi" --zip
generate "Mac" "$mac_app" --zip
generate "Mac.dmg" "$mac_dmg"
generate "Linux.AppImage" "$linux_appimg"
generate "Linux.deb" "$linux_deb"
