# AGENTS.md — `sbm-as`

Follow the universal Smart Boot Menu agent rules in [`smart-boot-menu/.github/AGENTS.md`](https://github.com/smart-boot-menu/.github/blob/main/AGENTS.md) (locally `../.github/AGENTS.md`).

## Assigned Repository & Role

- **Repository:** `smart-boot-menu/sbm-as`
- **Primary Dependencies & Specs:** `sbm-common`, `sbm-uefi` (`aarch64-unknown-uefi`), `sbm-core`, `docs/architecture-decisions/ADR-005`, `research/hardware/apple-silicon-m1-to-m5.md`

## Responsibilities (What You Own)

- Integrate upstream `AsahiLinux/m1n1` and `AsahiLinux/u-boot` (`asahi-releng`) so U-Boot (`CONFIG_EFI_LOADER`) executes the unmodified `aarch64-unknown-uefi` binary from `sbm-uefi`.
- Own Apple Silicon `PlatformCapabilities`, macOS reboot-target adapters, and permanent read-only protection rules for `iSCPreboot`, `RecoveryOS`, and macOS APFS containers.
- Maintain the `aarch64-apple` Fedora Asahi Minimal profile (< 900 MiB) and lawful local `asahi-fwextract` integration (never redistributing Apple firmware blobs).
- Lead external USB3 SSD boot validation (`EXP-AS-EXT01`) and track upstream M1/M2/M3 (supported) vs M4/M5 `T8142` (early bring-up) status honestly.

## Strict Boundaries (Do NOT Duplicate or Violate)

NEVER fork/duplicate `sbm-uefi`. NEVER modify the developer's M5 MacBook Air boot policy or partitions. NEVER redistribute Apple firmware blobs. NEVER claim M5 bare-metal support before upstream Asahi lands DART/PCIe/USB3.

## Engineering & Performance Targets

Keep total internal Apple Silicon stub + light Asahi recovery footprint around ~1.5–2.0 GiB so external USB3 SSDs can host user OSes.

*(Targets are goals, not claims; follow `../docs/testing/benchmarks.md` and archive real measurements in `../research/validation/evidence/`.)*

## Security, Testing & Interface Coordination

- Enforce all invariants in `../docs/threat-model/README.md` (offline-first, no secrets committed, bounded timeouts, safe storage plans).
- Run formatting, clippy, unit tests, and integration hooks before committing.
- Never silently diverge from `sbm-common` types or `docs/specifications/`; follow the Interface Change Protocol in `../.github/agent-reference.md`.
