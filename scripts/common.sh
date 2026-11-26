#!/bin/sh
# Shared guards for sbm-as scripts. Sourced, not executed.

refuse() {
    printf 'sbm-as: %s\n' "$1" >&2
    exit 2
}

# These scripts never partition a Mac, bless a volume, or enroll a boot object.
sbm_as_refuse_host_changes() {
    if [ "${SBM_AS_INSTALL:-}" = "1" ]; then
        refuse "SBM_AS_INSTALL is set; this repository does not install onto a Mac"
    fi
    for arg in "$@"; do
        case "$arg" in
            --install | install | bless | bputil | kmutil | diskutil)
                refuse "refusing command: $arg"
                ;;
            /dev/disk* | /dev/rdisk* | /Volumes/* | /boot/efi* | /System/*)
                refuse "refusing host path: $arg"
                ;;
        esac
    done
}

sbm_as_root() {
    cd -- "$(dirname -- "$1")/.." && pwd
}
