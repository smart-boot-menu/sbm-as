#!/bin/sh
# Boot the EFI probe under upstream U-Boot qemu_arm64.
# QEMU virt is not an Apple Silicon Mac. This script does not touch the host disk.
#
# Host tools, none of which are applied to the Mac's boot chain:
#   gmake (GNU make 4), Homebrew bash 5, Homebrew openssl@3, GnuTLS,
#   aarch64-elf-gcc, mtools, qemu-system-aarch64, rustc with aarch64-unknown-uefi.
# macOS /usr/bin/make is GNU make 3.81 and cannot parse this U-Boot tree.
# Homebrew's default openssl formula may be OpenSSL 4, whose ENGINE API is gone.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
# shellcheck disable=SC1091
. "$SCRIPT_DIR/common.sh"
sbm_as_refuse_host_changes "$@"

ROOT=$(sbm_as_root "$0")
# shellcheck disable=SC1091
. "$ROOT/config/pins.env"

BUILD="$ROOT/build"
SRC="$BUILD/upstream/u-boot-2026.10"
OBJ="$BUILD/u-boot-qemu"
POC="$BUILD/poc"
TARBALL="$BUILD/upstream/u-boot-v2026.10.tar.gz"
FRAGMENT="$ROOT/config/uboot-qemu-efi.fragment"
LOG="$POC/serial.log"

mkdir -p "$BUILD/upstream" "$POC"

if [ ! -f "$TARBALL" ]; then
    curl -fsSL -o "$TARBALL" "https://github.com/u-boot/u-boot/archive/refs/tags/${UPSTREAM_UBOOT_TAG}.tar.gz"
fi
got=$(shasum -a 256 "$TARBALL" | awk '{print $1}')
if [ "$got" != "$UPSTREAM_UBOOT_TARBALL_SHA256" ]; then
    refuse "U-Boot tarball sha256 is $got"
fi
if [ ! -f "$SRC/Makefile" ]; then
    tar -xzf "$TARBALL" -C "$BUILD/upstream"
fi

if ! command -v aarch64-elf-gcc >/dev/null 2>&1; then
    refuse "aarch64-elf-gcc is required to build U-Boot"
fi
MAKE=$(command -v gmake || true)
if [ -z "$MAKE" ]; then
    refuse "GNU make 4 (gmake) is required; the system make is too old for this U-Boot"
fi
QEMU=$(command -v qemu-system-aarch64 || true)
if [ -z "$QEMU" ]; then
    refuse "qemu-system-aarch64 is required"
fi

JOBS=$(sysctl -n hw.ncpu 2>/dev/null || printf '4')
OPENSSL=$(brew --prefix openssl@3 2>/dev/null || true)
if [ -z "$OPENSSL" ] || [ ! -f "$OPENSSL/include/openssl/evp.h" ]; then
    refuse "Homebrew openssl@3 is required; OpenSSL 4 removed the ENGINE API U-Boot still calls"
fi
HOMEBREW=$(brew --prefix)
BASH_BIN=$HOMEBREW/opt/bash/bin
if [ ! -x "$BASH_BIN/bash" ]; then
    refuse "Homebrew bash 5 is required; /bin/bash is too old for U-Boot's scripts"
fi
export PATH="$BASH_BIN:$PATH"
export PKG_CONFIG_PATH="$OPENSSL/lib/pkgconfig:$HOMEBREW/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export CPATH="$OPENSSL/include:$HOMEBREW/include${CPATH:+:$CPATH}"
export LIBRARY_PATH="$OPENSSL/lib:$HOMEBREW/lib${LIBRARY_PATH:+:$LIBRARY_PATH}"

"$MAKE" -C "$SRC" O="$OBJ" CROSS_COMPILE=aarch64-elf- qemu_arm64_defconfig
python3 - "$OBJ/.config" "$FRAGMENT" <<'PY'
import sys

config_path, fragment_path = sys.argv[1:]
lines = open(config_path, encoding="utf-8").read().splitlines()
fragment = []
keys = []
for raw in open(fragment_path, encoding="utf-8"):
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    fragment.append(line)
    if line.startswith("CONFIG_"):
        keys.append(line.split("=", 1)[0])
kept = []
for line in lines:
    drop = False
    for key in keys:
        if line.startswith(key + "=") or line.startswith("# " + key + " is not set"):
            drop = True
            break
    if not drop:
        kept.append(line)
kept.extend(fragment)
with open(config_path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(kept) + "\n")
PY
"$MAKE" -C "$SRC" O="$OBJ" CROSS_COMPILE=aarch64-elf- olddefconfig
grep -q '^CONFIG_BOOTDELAY=0$' "$OBJ/.config"
grep -q '^CONFIG_VIDEO_BOCHS=y$' "$OBJ/.config"
grep -q '^CONFIG_PREBOOT="usb start"$' "$OBJ/.config"
# Host tools pick up OpenSSL through the environment above. Drop objects
# built against a different OpenSSL so the link is not mixed.
rm -rf "$OBJ/tools"
"$MAKE" -C "$SRC" O="$OBJ" CROSS_COMPILE=aarch64-elf- -j"$JOBS"

if ! grep -q '^CONFIG_EFI_LOADER=y' "$OBJ/.config"; then
    refuse "U-Boot build did not enable CONFIG_EFI_LOADER"
fi

cargo build --manifest-path "$ROOT/Cargo.toml" --release \
    -p sbm-as-efi-probe --target aarch64-unknown-uefi --features uefi-app --bins

PROBE="$ROOT/target/aarch64-unknown-uefi/release/sbm-as-efi-probe.efi"
CHAIN="$ROOT/target/aarch64-unknown-uefi/release/sbm-as-chain-target.efi"
ESP="$POC/internal-esp.img"
EXT="$POC/external-esp.img"

make_fat() {
    image=$1
    rm -f "$image"
    dd if=/dev/zero of="$image" bs=1048576 count=32 status=none
    mformat -i "$image" -F -v "$2" ::
}

make_fat "$ESP" SBMESP
make_fat "$EXT" SBMEXT
mmd -i "$ESP" ::/EFI ::/EFI/BOOT ::/EFI/sbm
mmd -i "$EXT" ::/EFI ::/EFI/sbm
mcopy -i "$ESP" "$PROBE" ::/EFI/BOOT/BOOTAA64.EFI
mcopy -i "$ESP" "$CHAIN" ::/EFI/sbm/chain-target.efi
printf 'external\n' >"$POC/external-payload.efi"
mcopy -i "$EXT" "$POC/external-payload.efi" ::/EFI/sbm/external-payload.efi

helper=$(CDPATH= cd -- "$SCRIPT_DIR/../../sbm-lab/scripts" && pwd)/sbm-resources.sh
hold=$("$helper" reserve efi 1024 1 "$ROOT")
release_hold() {
    "$helper" release "$hold"
}
trap release_hold EXIT
export SBM_RESOURCES="$helper"
export SBM_RESERVATION="$hold"

python3 - "$LOG" "$QEMU" "$OBJ/u-boot.bin" "$ESP" "$EXT" <<'PY'
import os
import subprocess
import sys
import time

log_path, qemu, bios, esp, ext = sys.argv[1:]
attempts = [
    ["-machine", "virt,accel=hvf", "-cpu", "host"],
    ["-machine", "virt", "-cpu", "cortex-a57"],
]
markers = [
    "SBM-AS-PROBE: start",
    "SBM-AS-PROBE: qemu-virt-is-not-apple-silicon",
    "SBM-AS-PROBE: gop=",
    "SBM-AS-PROBE: keyboard=",
    "SBM-AS-PROBE: pointer=",
    "SBM-AS-PROBE: block-io=",
    "SBM-AS-PROBE: fdt=",
    "SBM-AS-PROBE: variable=",
    "SBM-AS-PROBE: policy-written",
    "SBM-AS-PROBE: discovered=\\EFI\\sbm\\chain-target.efi",
    "SBM-AS-CHAIN: ok",
    "SBM-AS-PROBE: chainload=ok",
    "SBM-AS-PROBE: discovered=\\EFI\\sbm\\external-payload.efi",
    "SBM-AS-PROBE: done",
]
chunks = []
for prefix in attempts:
    cmd = [
        qemu,
        *prefix,
        "-m",
        "1024",
        "-serial",
        "stdio",
        "-display",
        "none",
        "-device",
        "VGA",
        "-bios",
        bios,
        "-drive",
        f"if=virtio,format=raw,file={esp}",
        "-drive",
        "if=none,id=ext,format=raw,file=" + ext,
        "-device",
        "usb-ehci,id=ehci",
        "-device",
        "usb-storage,bus=ehci.0,drive=ext",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    helper = os.environ.get("SBM_RESOURCES")
    hold = os.environ.get("SBM_RESERVATION")
    if helper and hold:
        attached = subprocess.call([helper, "attach", hold, str(proc.pid)])
        if attached != 0:
            proc.kill()
            proc.wait(timeout=10)
            sys.exit(attached)
    lines = []
    deadline = time.time() + 90
    done = False
    while time.time() < deadline:
        line = proc.stdout.readline()
        if line == "" and proc.poll() is not None:
            break
        if line == "":
            time.sleep(0.05)
            continue
        sys.stdout.write(line)
        lines.append(line)
        if "SBM-AS-PROBE: done" in line:
            done = True
            break
    proc.kill()
    proc.wait(timeout=10)
    text = "".join(lines)
    chunks.append("SBM-AS-POC: command " + " ".join(cmd) + "\n" + text)
    with open(log_path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(chunks))
    if done and all(marker in text for marker in markers):
        print("SBM-AS-POC: pass", file=sys.stderr)
        sys.exit(0)
    if done:
        missing = [marker for marker in markers if marker not in text]
        print("SBM-AS-POC: incomplete " + ", ".join(missing), file=sys.stderr)
        sys.exit(1)
print("SBM-AS-POC: probe did not finish", file=sys.stderr)
sys.exit(1)
PY
