# Apple Silicon implementation plan (SBM 0.1)

This is the plan for a maintainable path onto Apple Silicon. It does not install anything on the development Mac, and it does not change that Mac's boot policy or partitions.

## Keep

- Reuse upstream `m1n1` and `AsahiLinux/u-boot` (`asahi-releng`). Do not fork them.
- Run the same `aarch64-unknown-uefi` `sbm-uefi` binary U-Boot already uses for GRUB. Do not write a second boot manager.
- Persist SBM's own boot choice in an ESP file. Asahi documents that this platform has no UEFI variable store, so `BootOrder` is not a configuration channel.
- Classify Apple `iBootSystemContainer`, `RecoveryOSContainer`, and macOS APFS volumes as protected before any storage plan can touch them.
- Treat macOS as an Apple reboot target. Do not `StartImage` macOS.

## Prove without the hardware

- Build upstream U-Boot `v2026.10` `qemu_arm64` and boot a small EFI probe plus a second EFI image.
- The probe checks GOP, text input, pointer, block I/O, FAT discovery, an ESP policy file, EFI variable writes, the FDT configuration table, and `LoadImage`/`StartImage`.
- State in the log and the docs that `qemu virt` is not an Apple Silicon Mac.

## Leave for upstream

- Physical M4 and M5 boot, including this M5 development machine.
- A built Fedora Asahi image. `sbm-core` keeps one mkosi pipeline; this repo only records the Apple profile differences.
- Native installation. The installer document tells the future install agent to reuse the Asahi 1TR flow and not to invent a boot-policy writer.

## Non-goals

- Bypassing Boot ROM, LLB, LocalPolicy, or paired recoveryOS.
- Redistributing Apple firmware.
- Claiming that a generic ARM64 UEFI disk, Windows, or an x86 image boots natively on these machines.
