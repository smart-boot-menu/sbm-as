#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod boot;
pub mod external;
pub mod partition;
pub mod profile;
pub mod recovery;

/// ESP path of the boot-policy file used when EFI variables do not persist.
pub const ESP_POLICY_FILE: &str = "\\EFI\\sbm\\boot-policy.json";

/// Second EFI image the U-Boot proof-of-concept chainloads.
pub const ESP_CHAIN_TARGET_FILE: &str = "\\EFI\\sbm\\chain-target.efi";

/// Marker file placed on the virtual external disk in the QEMU proof of concept.
pub const ESP_EXTERNAL_MARKER_FILE: &str = "\\EFI\\sbm\\external-payload.efi";

/// QEMU's `virt` machine running `qemu_arm64` U-Boot is not an Apple Silicon Mac.
pub const QEMU_ARM64_MODELS_APPLE_SILICON: bool = false;

/// SBM does not bypass Apple Boot ROM, LLB, LocalPolicy, or paired recoveryOS.
pub const BYPASSES_APPLE_BOOT_SECURITY: bool = false;
