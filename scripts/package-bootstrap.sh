#!/bin/sh
# Write a bootstrap manifest for the internal Apple Silicon stub.
# This does not create boot.bin on a disk and does not call bless, bputil, or kmutil.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck disable=SC1091
. "$SCRIPT_DIR/common.sh"
sbm_as_refuse_host_changes "$@"

ROOT=$(sbm_as_root "$0")
# shellcheck disable=SC1091
. "$ROOT/config/pins.env"

OUT="$ROOT/build/manifests"
mkdir -p "$OUT"
MANIFEST="$OUT/bootstrap-manifest.json"

cat >"$MANIFEST" <<EOF
{
  "schema": "sbm-as-bootstrap",
  "version": 1,
  "installed": false,
  "qemu_models_apple_silicon": false,
  "pins": {
    "m1n1": "$M1N1_COMMIT",
    "m1n1_tag": "$M1N1_TAG",
    "asahi_u_boot": "$ASAHI_UBOOT_COMMIT",
    "asahi_u_boot_base": "$ASAHI_UBOOT_BASE",
    "linux_asahi": "$LINUX_ASAHI_COMMIT",
    "linux_asahi_tag": "$LINUX_ASAHI_TAG",
    "asahi_installer": "$ASAHI_INSTALLER_COMMIT",
    "uefi_crate": "$UEFI_CRATE"
  },
  "stage2_recipe": [
    "m1n1.bin from the pinned m1n1 tree",
    "device trees built from the pinned linux-asahi tree",
    "gzip of u-boot-nodtb.bin from Asahi U-Boot apple_m1_defconfig plus config/apple-uboot.fragment"
  ],
  "efi_payload": "\\\\EFI\\\\BOOT\\\\BOOTAA64.EFI is the unmodified aarch64-unknown-uefi sbm-uefi binary",
  "policy_file": "\\\\EFI\\\\sbm\\\\boot-policy.json",
  "refuses": ["bless", "bputil", "kmutil", "diskutil", "host disk devices"]
}
EOF

printf 'wrote %s\n' "$MANIFEST"
printf 'no host disk was modified\n'
