# Apple Silicon bootstrap (`sbm-as`)

Platform adapter, partition guardrails, and packaging scripts for running Smart Boot Menu on Apple Silicon. The boot chain stays Apple's, then upstream m1n1, then Asahi U-Boot. The menu itself is the shared aarch64 UEFI binary from `sbm-uefi`. This repository does not replace that binary and does not install anything on the development Mac.

**Status:** The capability adapter and the QEMU U-Boot EFI probe are in place. Physical M4 and M5 boot is blocked upstream. M5 `T8142` is not an Asahi install target. QEMU `virt` is not an Apple Silicon Mac.

## What is here

- `crates/sbm-as-platform` — per-SoC profile, external-boot levels, macOS reboot target, and Apple partition classes. It fills the existing `PlatformCapabilities` fields and does not change `sbm-common`.
- `crates/sbm-as-efi-probe` — a small `aarch64-unknown-uefi` program (`uefi` 0.41.0) that reports GOP, input, block devices, the device tree, variable behavior, an ESP policy file, and a `StartImage` of a second EFI application. It is not the menu.
- `config/pins.env` — m1n1 `v2.0.1`, Asahi U-Boot `asahi-releng` `ec49c9d`, linux-asahi `v7.3.4`, asahi-installer `0259acb`, upstream U-Boot `v2026.10`.
- `config/uboot-qemu-efi.fragment` — applied only to the QEMU build.
- `config/apple-uboot.fragment` — untested `usb start` preboot for a future hardware build. The proof script does not apply it to a Mac.
- `scripts/uboot-efi-poc.sh` — reproducible QEMU probe. Refuses install, bless, and host disk paths.
- `scripts/package-bootstrap.sh` — writes a manifest of the stage-2 recipe. It does not install it.
- `scripts/fetch-upstream.sh` — checks out the pinned sources under `build/`, which is gitignored.

## Proof boundary

`scripts/uboot-efi-poc.sh` builds upstream U-Boot `v2026.10` for `qemu_arm64` and boots the probe. The passing run on 2026-11-26 used TCG (`-machine virt -cpu cortex-a57`) after HVF aborted. The log is summarized in `research/validation/evidence/sbm-as-uboot-efi-2026-11-26.md`. That machine has virtio and USB EHCI. It does not show Apple NVMe, DART, or xHCI.

Host tools for that script: GNU make 4 (`gmake`), Homebrew bash 5, Homebrew `openssl@3`, GnuTLS, `aarch64-elf-gcc`, mtools, and `qemu-system-aarch64`. macOS `/usr/bin/make` is too old for this U-Boot tree.

## Documents

- Architecture: [`docs` boot architecture](https://github.com/smart-boot-menu/docs/blob/main/hardware/apple-silicon-boot-architecture.md) and [ADR-005](https://github.com/smart-boot-menu/docs/blob/main/architecture-decisions/ADR-005-apple-silicon-m1n1-uboot-strategy.md)
- Research: [M1–M5 matrix](https://github.com/smart-boot-menu/research/blob/main/hardware/apple-silicon-m1-to-m5.md), [external boot](https://github.com/smart-boot-menu/research/blob/main/hardware/apple-silicon-external-boot.md)
- Handoff: [HANDOFF.md](HANDOFF.md)
- Agent guide: [AGENTS.md](AGENTS.md)

## License

Original code and documentation in this repository are dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), subject to third-party licensing requirements for bundled external components.
