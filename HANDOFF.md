# Handoff — Apple Silicon

**Date:** 2026-11-26

The milestone is a maintainable path and a QEMU protocol probe. It is not physical M5 support, and it is not a claim that SBM can boot an OS the hardware cannot boot.

## Interface

No `sbm-common` change. `PlatformCapabilities` already has the fields the adapter fills. `nvram_writable: false` is the wire signal that UEFI variables do not persist on Apple Silicon. `can_chainload_efi` means U-Boot is running; `SocProfile::bridge_reachable` is the extra check for whether this SoC can get there. Callers must use both.

## What landed

- `crates/sbm-as-platform` — SoC profiles from the Asahi tables fetched 2026-11-26, six external-boot levels, macOS reboot target, Apple partition classes.
- `crates/sbm-as-efi-probe` — `uefi` 0.41.0 probe and chain target. This is not `sbm-uefi` and not a boot manager.
- Pins, the QEMU fragment, the untested Apple `usb start` fragment, and `scripts/uboot-efi-poc.sh`, `scripts/package-bootstrap.sh`, `scripts/fetch-upstream.sh`.
- Corrected research, ADR-005 amendment, and `docs/hardware/apple-silicon-boot-architecture.md`.
- `sbm-core/docs/apple-silicon-profile.md` — differences for the shared mkosi pipeline. No image was built.
- `sbm-lab/scenarios/exp-as-ext01-phase1.md` and `scripts/run-uboot-efi-probe.sh`.
- `install/docs/apple-silicon.md` — workflow for a future installer. It was not executed.

## Probe result

`scripts/uboot-efi-poc.sh` passed under QEMU `virt` / TCG `cortex-a57` / U-Boot `v2026.10`. HVF aborted with `hvf_handle_exception` before U-Boot started. The probe reported GOP 1024x768 with a successful 32x32 fill, a keyboard, no pointer, an ESP policy write, `StartImage` of the chain target, and the marker file on the EHCI disk. Evidence: `research/validation/evidence/sbm-as-uboot-efi-2026-11-26.md`.

Same-boot `SetVariable(NON_VOLATILE)` succeeded on this QEMU build, and U-Boot printed that it cannot persist variables without a system partition. That does not contradict Asahi's "no variable store" rule. The Apple adapter still sets `nvram_writable` false.

## Blockers

- M3 Ultra, every M4 device, and every M5 device including `T8142` have Asahi installer `no`. M4 DART is `TBA` even where USB3 is `linux-asahi`. M5 USB, Thunderbolt, PCIe, DART, and GPU are `TBA`.
- `sbm-uefi` does not yet produce the shared `BOOTAA64.EFI`. The probe only covers the protocols that binary will need.
- `config/apple-uboot.fragment` has not been built into Asahi U-Boot or run on hardware. The pinned defconfig still boots with `bootflow scan -b` and does not start USB first.
- No native install was run. Image sizes in the ADRs are targets, not measurements. Apple firmware is not redistributed.
- CI checks out `sbm-common` as a sibling. The workflow token has to be able to read that repository.

## Safety

The development Mac's internal storage, boot configuration, and security policy were not modified. The proof script refuses install, bless, and host disk paths.
