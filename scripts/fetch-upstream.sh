#!/bin/sh
# Fetch pinned m1n1, Asahi U-Boot, linux-asahi, and the Asahi installer into build/.
# Does not build them and does not write outside this repository.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck disable=SC1091
. "$SCRIPT_DIR/common.sh"
sbm_as_refuse_host_changes "$@"

ROOT=$(sbm_as_root "$0")
# shellcheck disable=SC1091
. "$ROOT/config/pins.env"

BUILD="$ROOT/build/upstream"
mkdir -p "$BUILD"

fetch_commit() {
    name=$1
    url=$2
    commit=$3
    dest="$BUILD/$name"
    if [ -d "$dest/.git" ]; then
        git -C "$dest" fetch --depth 1 origin "$commit"
    else
        rm -rf "$dest"
        mkdir -p "$dest"
        git -C "$dest" init --quiet
        git -C "$dest" remote add origin "$url"
        git -C "$dest" fetch --depth 1 origin "$commit"
    fi
    git -C "$dest" checkout --detach --quiet FETCH_HEAD
    got=$(git -C "$dest" rev-parse HEAD)
    if [ "$got" != "$commit" ]; then
        refuse "$name checked out $got, expected $commit"
    fi
    printf 'pinned %s %s\n' "$name" "$got"
}

fetch_commit m1n1 "$M1N1_URL" "$M1N1_COMMIT"
fetch_commit asahi-u-boot "$ASAHI_UBOOT_URL" "$ASAHI_UBOOT_COMMIT"
fetch_commit linux-asahi "$LINUX_ASAHI_URL" "$LINUX_ASAHI_COMMIT"
fetch_commit asahi-installer "$ASAHI_INSTALLER_URL" "$ASAHI_INSTALLER_COMMIT"

printf 'fetch complete under %s\n' "$BUILD"
