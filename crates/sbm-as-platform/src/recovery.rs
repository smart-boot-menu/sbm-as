//! Differences the shared `sbm-core` mkosi pipeline needs for Apple Silicon.
//!
//! This is not a second distribution. Image size is a target until a build
//! records a measurement.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppleRecoveryRequirements {
    pub page_size_bytes: u32,
    pub kernel: &'static str,
    pub redistribute_apple_firmware: bool,
    pub graphics: &'static str,
    pub storage: &'static str,
    pub networking: &'static str,
    pub input: &'static str,
    pub boot_parameters: &'static str,
    pub device_tree: &'static str,
    pub packaging: &'static str,
}

pub fn apple_recovery_requirements() -> AppleRecoveryRequirements {
    AppleRecoveryRequirements {
        page_size_bytes: 16 * 1024,
        kernel: "linux-asahi built for 16 KiB pages. Fedora Asahi's package name is kernel-16k; confirm the NEVRA when the mkosi build runs.",
        redistribute_apple_firmware: false,
        graphics: "Boot with the framebuffer U-Boot preserved. Use mesa-asahi only on SoCs whose GPU row is linux-asahi (M1 and M2). Elsewhere use the framebuffer or llvmpipe. UEFI GOP is not a GPU.",
        storage: "Internal NVMe uses the Apple NVMe driver and 4 KiB logical blocks. Do not mount or write iBootSystemContainer, RecoveryOSContainer, or macOS APFS volumes from the recovery image.",
        networking: "The Apple U-Boot defconfig sets CONFIG_NO_NET, so there is no UEFI SNP. Wi-Fi and Bluetooth need firmware extracted on the machine by asahi-fwextract.",
        input: "U-Boot enables the SPI and MTP keyboards and USB HID keyboards. It does not enable the internal trackpad. The recovery image needs the Linux keyboard and trackpad drivers.",
        boot_parameters: "Pass the FDT from m1n1 through U-Boot. Do not boot an ACPI-only generic ARM64 kernel. Keep the root on the verity image the shared pipeline already builds.",
        device_tree: "One OS owns /m1n1/boot.bin on the internal System ESP, including the device trees. Other installs set M1N1_UPDATE_DISABLED=1. Mismatched trees are a known boot break.",
        packaging: "Select this profile from the shared mkosi pipeline. Do not keep a separate rootfs build. Extract vendor firmware at install time into the System ESP and do not ship those blobs.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ESP_CHAIN_TARGET_FILE, ESP_POLICY_FILE, QEMU_ARM64_MODELS_APPLE_SILICON};

    #[test]
    fn recovery_profile_keeps_sixteen_kib_pages_and_local_firmware() {
        let requirements = apple_recovery_requirements();
        assert_eq!(requirements.page_size_bytes, 16384);
        assert!(!requirements.redistribute_apple_firmware);
        const { assert!(!QEMU_ARM64_MODELS_APPLE_SILICON) };
        assert!(ESP_POLICY_FILE.starts_with("\\EFI\\sbm\\"));
        assert!(ESP_CHAIN_TARGET_FILE.ends_with(".efi"));
    }

    #[test]
    fn efi_probe_sources_use_the_shared_esp_paths() {
        let probe = include_str!("../../sbm-as-efi-probe/src/probe.rs");
        assert!(probe.contains(ESP_POLICY_FILE));
        assert!(probe.contains(ESP_CHAIN_TARGET_FILE));
        let chain = include_str!("../../sbm-as-efi-probe/src/chain.rs");
        assert!(chain.contains("SBM-AS-CHAIN: ok"));
    }
}
