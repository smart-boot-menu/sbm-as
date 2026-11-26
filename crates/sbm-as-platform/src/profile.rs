//! Static, generation-specific view of upstream Apple Silicon support.
//!
//! Runtime code must prefer what the firmware actually published. These
//! profiles only stop SBM from offering an action the upstream tables do not
//! support. They are transcribed from the Asahi feature tables fetched on
//! 2026-11-26, not from a boot of this development machine.

use sbm_types::{CpuArch, FirmwarePlatform, PlatformCapabilities, SecureBootState};

/// Marketing generation. Support is not uniform inside a generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppleGeneration {
    M1,
    M2,
    M3,
    M4,
    M5,
}

/// SoC identifiers used by Asahi's feature tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocId {
    T8103,
    T6000,
    T6001,
    T6002,
    T8112,
    T6020,
    T6021,
    T6022,
    T8122,
    T6030,
    T6031,
    T6034,
    T6032,
    T8140,
    T8132,
    T6040,
    T6041,
    T8142,
    T6050,
    /// Some M5 Max boards in the Asahi device list. The feature table does
    /// not give this id its own column; those boards sit in the Pro/Max/Ultra
    /// table, which is still TBA.
    T6051,
}

/// How far an upstream feature has been published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureState {
    /// Merged in a released mainline kernel.
    Upstream(&'static str),
    /// Present in `linux-asahi`. `target` is the upstream release Asahi names.
    LinuxAsahi {
        target: Option<&'static str>,
    },
    Wip,
    Tba,
    NotPresent,
}

/// Official Asahi installer coverage for this SoC, before device exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerStatus {
    Yes,
    No,
    Wip,
}

/// A machine whose installer status is narrower than its SoC row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceException {
    /// OpenBSD and Asahi both still mark this machine short of the other M2s.
    MacPro2023,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocProfile {
    pub soc: SocId,
    pub generation: AppleGeneration,
    pub installer: InstallerStatus,
    pub usb3: FeatureState,
    pub dart: FeatureState,
    pub nvme: FeatureState,
    pub gpu: FeatureState,
    pub wifi: FeatureState,
    pub boot_framebuffer: FeatureState,
    pub internal_keyboard: FeatureState,
    pub internal_touchpad: FeatureState,
}

impl SocId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::T8103 => "T8103",
            Self::T6000 => "T6000",
            Self::T6001 => "T6001",
            Self::T6002 => "T6002",
            Self::T8112 => "T8112",
            Self::T6020 => "T6020",
            Self::T6021 => "T6021",
            Self::T6022 => "T6022",
            Self::T8122 => "T8122",
            Self::T6030 => "T6030",
            Self::T6031 => "T6031",
            Self::T6034 => "T6034",
            Self::T6032 => "T6032",
            Self::T8140 => "T8140",
            Self::T8132 => "T8132",
            Self::T6040 => "T6040",
            Self::T6041 => "T6041",
            Self::T8142 => "T8142",
            Self::T6050 => "T6050",
            Self::T6051 => "T6051",
        }
    }
}

const ALL_SOCS: &[SocId] = &[
    SocId::T8103,
    SocId::T6000,
    SocId::T6001,
    SocId::T6002,
    SocId::T8112,
    SocId::T6020,
    SocId::T6021,
    SocId::T6022,
    SocId::T8122,
    SocId::T6030,
    SocId::T6031,
    SocId::T6034,
    SocId::T6032,
    SocId::T8140,
    SocId::T8132,
    SocId::T6040,
    SocId::T6041,
    SocId::T8142,
    SocId::T6050,
    SocId::T6051,
];

pub fn all_socs() -> &'static [SocId] {
    ALL_SOCS
}

pub fn soc_profile(soc: SocId) -> SocProfile {
    let mut profile = match soc {
        SocId::T8103 => m1_base(),
        SocId::T6000 | SocId::T6001 | SocId::T6002 => m1_pro(),
        SocId::T8112 => m2_base(),
        SocId::T6020 | SocId::T6021 | SocId::T6022 => m2_pro(),
        SocId::T8122 => m3_base(),
        SocId::T6030 | SocId::T6031 | SocId::T6034 => m3_pro(),
        SocId::T6032 => m3_ultra(),
        SocId::T8140 => m4_a18(),
        SocId::T8132 => m4_base(),
        SocId::T6040 | SocId::T6041 => m4_pro(),
        SocId::T8142 => m5_base(),
        SocId::T6050 | SocId::T6051 => m5_pro(),
    };
    profile.soc = soc;
    profile
}

pub fn profile_for_device(soc: SocId, device: Option<DeviceException>) -> SocProfile {
    let mut profile = soc_profile(soc);
    let m2_pro_family = matches!(soc, SocId::T6020 | SocId::T6021 | SocId::T6022);
    if device == Some(DeviceException::MacPro2023) && m2_pro_family {
        profile.installer = InstallerStatus::Wip;
    }
    profile
}

impl SocProfile {
    /// USB mass storage is far enough along that an external disk is a
    /// supported boot target for the capability bit.
    ///
    /// The bit stays false when the installer does not support the SoC, when
    /// DART is still `WIP` or `TBA`, or when USB3 itself is not published.
    /// A `linux-asahi` USB cell alone is not enough: M4 base is the case
    /// where USB is marked `linux-asahi` and DART is still `TBA`.
    pub fn external_usb_boot_supported(self) -> bool {
        self.installer == InstallerStatus::Yes
            && feature_published(self.usb3)
            && !matches!(self.dart, FeatureState::Wip | FeatureState::Tba)
    }

    /// Project the shared wire capabilities for the Apple boot bridge.
    ///
    /// `can_chainload_efi` describes U-Boot once it is running. It does not
    /// mean this SoC can reach U-Boot today. Callers that offer installation
    /// must also check [`SocProfile::bridge_reachable`].
    pub fn capabilities_if_bridge_is_up(self) -> PlatformCapabilities {
        PlatformCapabilities {
            arch: CpuArch::Aarch64,
            firmware: FirmwarePlatform::AppleSiliconM1n1Uboot,
            page_size_bytes: 16 * 1024,
            secure_boot_state: SecureBootState::Unsupported,
            tpm_present: false,
            nvram_writable: false,
            gop_framebuffer: feature_published(self.boot_framebuffer),
            uefi_networking: false,
            can_chainload_efi: true,
            can_direct_chainload_macos: false,
            external_usb_boot_supported: self.external_usb_boot_supported(),
            ram_total_mib: None,
        }
    }

    pub fn bridge_reachable(self) -> bool {
        self.installer == InstallerStatus::Yes && feature_published(self.nvme)
    }
}

fn feature_published(state: FeatureState) -> bool {
    matches!(
        state,
        FeatureState::Upstream(_) | FeatureState::LinuxAsahi { .. }
    )
}

fn m1_base() -> SocProfile {
    SocProfile {
        soc: SocId::T8103,
        generation: AppleGeneration::M1,
        installer: InstallerStatus::Yes,
        usb3: FeatureState::Upstream("7.0"),
        dart: FeatureState::Upstream("5.15"),
        nvme: FeatureState::Upstream("5.19"),
        gpu: FeatureState::LinuxAsahi { target: None },
        wifi: FeatureState::Upstream("6.1"),
        boot_framebuffer: FeatureState::Upstream("5.17"),
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::LinuxAsahi { target: None },
    }
}

fn m1_pro() -> SocProfile {
    SocProfile {
        soc: SocId::T6000,
        generation: AppleGeneration::M1,
        installer: InstallerStatus::Yes,
        usb3: FeatureState::Upstream("7.0"),
        dart: FeatureState::Upstream("6.1"),
        nvme: FeatureState::Upstream("5.19"),
        gpu: FeatureState::LinuxAsahi { target: None },
        wifi: FeatureState::Upstream("6.1"),
        boot_framebuffer: FeatureState::Upstream("5.17"),
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::LinuxAsahi { target: None },
    }
}

fn m2_base() -> SocProfile {
    SocProfile {
        soc: SocId::T8112,
        generation: AppleGeneration::M2,
        installer: InstallerStatus::Yes,
        usb3: FeatureState::Upstream("7.0"),
        dart: FeatureState::Upstream("6.3"),
        nvme: FeatureState::Upstream("5.19"),
        gpu: FeatureState::LinuxAsahi { target: None },
        wifi: FeatureState::Upstream("6.1"),
        boot_framebuffer: FeatureState::LinuxAsahi { target: None },
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::LinuxAsahi { target: None },
    }
}

fn m2_pro() -> SocProfile {
    let mut profile = m2_base();
    profile.soc = SocId::T6020;
    profile.dart = FeatureState::Upstream("6.18");
    profile.wifi = FeatureState::LinuxAsahi { target: None };
    profile
}

fn m3_base() -> SocProfile {
    SocProfile {
        soc: SocId::T8122,
        generation: AppleGeneration::M3,
        installer: InstallerStatus::Yes,
        usb3: FeatureState::LinuxAsahi { target: None },
        dart: FeatureState::LinuxAsahi { target: None },
        nvme: FeatureState::LinuxAsahi {
            target: Some("7.5"),
        },
        gpu: FeatureState::Wip,
        wifi: FeatureState::LinuxAsahi { target: None },
        boot_framebuffer: FeatureState::LinuxAsahi { target: None },
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::LinuxAsahi { target: None },
    }
}

fn m3_pro() -> SocProfile {
    let mut profile = m3_base();
    profile.soc = SocId::T6030;
    profile.gpu = FeatureState::Tba;
    profile
}

fn m3_ultra() -> SocProfile {
    let mut profile = m3_pro();
    profile.soc = SocId::T6032;
    profile.installer = InstallerStatus::No;
    profile.usb3 = FeatureState::Wip;
    profile.boot_framebuffer = FeatureState::Wip;
    profile
}

fn m4_common(soc: SocId) -> SocProfile {
    SocProfile {
        soc,
        generation: AppleGeneration::M4,
        installer: InstallerStatus::No,
        usb3: FeatureState::Tba,
        dart: FeatureState::Tba,
        nvme: FeatureState::LinuxAsahi { target: None },
        gpu: FeatureState::Tba,
        wifi: FeatureState::LinuxAsahi { target: None },
        boot_framebuffer: FeatureState::LinuxAsahi {
            target: Some("7.4"),
        },
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::Tba,
    }
}

fn m4_a18() -> SocProfile {
    let mut profile = m4_common(SocId::T8140);
    profile.wifi = FeatureState::Tba;
    profile.boot_framebuffer = FeatureState::LinuxAsahi {
        target: Some("7.4"),
    };
    profile
}

fn m4_base() -> SocProfile {
    let mut profile = m4_common(SocId::T8132);
    profile.usb3 = FeatureState::LinuxAsahi { target: None };
    profile
}

fn m4_pro() -> SocProfile {
    let mut profile = m4_common(SocId::T6040);
    profile.boot_framebuffer = FeatureState::LinuxAsahi {
        target: Some("7.5"),
    };
    profile
}

fn m5_base() -> SocProfile {
    SocProfile {
        soc: SocId::T8142,
        generation: AppleGeneration::M5,
        installer: InstallerStatus::No,
        usb3: FeatureState::Tba,
        dart: FeatureState::Tba,
        nvme: FeatureState::LinuxAsahi { target: None },
        gpu: FeatureState::Tba,
        wifi: FeatureState::Tba,
        boot_framebuffer: FeatureState::LinuxAsahi {
            target: Some("7.5"),
        },
        internal_keyboard: FeatureState::LinuxAsahi { target: None },
        internal_touchpad: FeatureState::Tba,
    }
}

fn m5_pro() -> SocProfile {
    SocProfile {
        soc: SocId::T6050,
        generation: AppleGeneration::M5,
        installer: InstallerStatus::No,
        usb3: FeatureState::Tba,
        dart: FeatureState::Tba,
        nvme: FeatureState::Tba,
        gpu: FeatureState::Tba,
        wifi: FeatureState::Tba,
        boot_framebuffer: FeatureState::Tba,
        internal_keyboard: FeatureState::Tba,
        internal_touchpad: FeatureState::Tba,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn every_soc_has_a_profile_and_sixteen_kib_pages() {
        for soc in all_socs() {
            let profile = soc_profile(*soc);
            assert_eq!(profile.soc, *soc);
            let caps = profile.capabilities_if_bridge_is_up();
            assert_eq!(caps.page_size_bytes, 16384);
            assert!(!caps.nvram_writable);
            assert!(!caps.can_direct_chainload_macos);
            assert_eq!(caps.secure_boot_state, SecureBootState::Unsupported);
            assert!(!caps.uefi_networking);
            assert!(!caps.tpm_present);
        }
    }

    #[test]
    fn usb_capability_follows_installer_and_dart_not_a_single_usb_cell() {
        assert!(soc_profile(SocId::T8103).external_usb_boot_supported());
        assert!(soc_profile(SocId::T8112).external_usb_boot_supported());
        assert!(soc_profile(SocId::T8122).external_usb_boot_supported());
        assert!(soc_profile(SocId::T6030).external_usb_boot_supported());
        assert!(!soc_profile(SocId::T6032).external_usb_boot_supported());
        assert!(!soc_profile(SocId::T8132).external_usb_boot_supported());
        assert_eq!(
            soc_profile(SocId::T8132).usb3,
            FeatureState::LinuxAsahi { target: None }
        );
        assert_eq!(soc_profile(SocId::T8132).dart, FeatureState::Tba);
        assert!(!soc_profile(SocId::T8142).external_usb_boot_supported());
        assert!(!soc_profile(SocId::T6050).external_usb_boot_supported());
    }

    #[test]
    fn m5_base_is_not_an_install_target_and_pro_has_no_nvme_row() {
        let air = soc_profile(SocId::T8142);
        assert!(!air.bridge_reachable());
        assert_eq!(air.wifi, FeatureState::Tba);
        assert_eq!(air.nvme, FeatureState::LinuxAsahi { target: None });
        assert_eq!(soc_profile(SocId::T6050).nvme, FeatureState::Tba);
        assert!(
            !soc_profile(SocId::T6050)
                .capabilities_if_bridge_is_up()
                .gop_framebuffer
        );
    }

    #[test]
    fn mac_pro_2023_installer_is_not_the_m2_default() {
        let soc = soc_profile(SocId::T6020);
        assert_eq!(soc.installer, InstallerStatus::Yes);
        let device = profile_for_device(SocId::T6020, Some(DeviceException::MacPro2023));
        assert_eq!(device.installer, InstallerStatus::Wip);
        assert!(!device.bridge_reachable());
    }

    #[test]
    fn m4_wifi_is_published_and_touchpad_is_not() {
        let m4 = soc_profile(SocId::T8132);
        assert_eq!(m4.wifi, FeatureState::LinuxAsahi { target: None });
        assert_eq!(
            soc_profile(SocId::T6040).wifi,
            FeatureState::LinuxAsahi { target: None }
        );
        assert_eq!(soc_profile(SocId::T8140).wifi, FeatureState::Tba);
        assert_eq!(m4.internal_touchpad, FeatureState::Tba);
        assert_eq!(m4.installer, InstallerStatus::No);
    }

    #[test]
    fn m5_max_t6051_follows_the_pro_max_ultra_table() {
        assert_eq!(soc_profile(SocId::T6051).nvme, FeatureState::Tba);
        assert!(!soc_profile(SocId::T6051).bridge_reachable());
    }

    #[test]
    fn soc_ids_are_unique() {
        let mut names = Vec::new();
        for soc in all_socs() {
            names.push(soc.as_str());
        }
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), all_socs().len());
    }
}
