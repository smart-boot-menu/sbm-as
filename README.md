# Apple Silicon Bootstrap & Platform Integration (`sbm-as`)

Apple Silicon platform integration (`m1n1` + `AsahiLinux/u-boot` -> shared `aarch64` `sbm-uefi`), light Asahi `sbm-core` profile integration, Apple partition protection guardrails, and external-storage boot research.

> **Project Status:** Phase 0 initialized — ready for dedicated engineering agent implementation targeting **SBM 0.1**.

## Scope & Responsibilities

- Integrate upstream `AsahiLinux/m1n1` and `AsahiLinux/u-boot` (`asahi-releng`) so U-Boot (`CONFIG_EFI_LOADER`) executes the unmodified `aarch64-unknown-uefi` binary from `sbm-uefi`.
- Own Apple Silicon `PlatformCapabilities`, macOS reboot-target adapters, and permanent read-only protection rules for `iSCPreboot`, `RecoveryOS`, and macOS APFS containers.
- Maintain the `aarch64-apple` Fedora Asahi Minimal profile (< 900 MiB) and lawful local `asahi-fwextract` integration (never redistributing Apple firmware blobs).
- Lead external USB3 SSD boot validation (`EXP-AS-EXT01`) and track upstream M1/M2/M3 (supported) vs M4/M5 `T8142` (early bring-up) status honestly.

## Canonical Documentation & Contracts

- Central coordination & roadmap: [`smart-boot-menu/sbm`](https://github.com/smart-boot-menu/sbm)
- Normative specifications & ADRs: [`smart-boot-menu/docs`](https://github.com/smart-boot-menu/docs)
- Primary-source technical research: [`smart-boot-menu/research`](https://github.com/smart-boot-menu/research)
- Shared Rust types & JSON Schemas: [`smart-boot-menu/sbm-common`](https://github.com/smart-boot-menu/sbm-common)
- Agent guide: [AGENTS.md](AGENTS.md)

## License

Original code and documentation in this repository are dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE), subject to third-party licensing requirements for bundled external components.
