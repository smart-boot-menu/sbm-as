//! Boot-target decisions for the Apple bridge.
//!
//! The adapter decides whether an entry is an EFI `StartImage`, an Apple
//! reboot, or a rejection. It does not talk to firmware.

use alloc::string::String;

use sbm_types::{
    BootTarget, BootTargetId, BootTargetKind, CpuArch, FirmwarePlatform, OsFamily,
    PlatformCapabilities,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    InvalidVolumeGroupId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainloadDecision {
    AllowEfi,
    RebootToApple,
    Reject { reason: &'static str },
}

pub fn macos_reboot_target(
    volume_group_uuid: &str,
    display_name: &str,
    removable: bool,
) -> Result<BootTarget, AdapterError> {
    let uuid = volume_group_uuid.trim().to_ascii_lowercase();
    if !is_uuid(&uuid) {
        return Err(AdapterError::InvalidVolumeGroupId);
    }
    let mut id = String::from("apple-macos:");
    id.push_str(&uuid);
    Ok(BootTarget {
        id: BootTargetId(id),
        display_name: String::from(display_name),
        os_family: OsFamily::MacOs,
        kind: BootTargetKind::AppleMacOsReboot,
        arch: CpuArch::Aarch64,
        partition_guid: None,
        efi_path: None,
        kernel_cmdline: None,
        icon_hint: None,
        removable_media: removable,
        hidden: false,
        verified_signature: None,
    })
}

pub fn chainload_decision(caps: &PlatformCapabilities, target: &BootTarget) -> ChainloadDecision {
    if caps.firmware == FirmwarePlatform::AppleSiliconM1n1Uboot
        && (target.kind == BootTargetKind::AppleMacOsReboot || target.os_family == OsFamily::MacOs)
    {
        return ChainloadDecision::RebootToApple;
    }
    if caps.firmware == FirmwarePlatform::AppleSiliconM1n1Uboot
        && target.os_family == OsFamily::Windows
    {
        return ChainloadDecision::Reject {
            reason: "Windows is not a native Apple Silicon boot target",
        };
    }
    if target.arch != caps.arch {
        return ChainloadDecision::Reject {
            reason: "target architecture does not match the firmware",
        };
    }
    if !caps.can_chainload_efi {
        return ChainloadDecision::Reject {
            reason: "firmware cannot start an EFI image",
        };
    }
    match target.kind {
        BootTargetKind::EfiBinary
        | BootTargetKind::BlsType1
        | BootTargetKind::BlsUki
        | BootTargetKind::SbmCore => ChainloadDecision::AllowEfi,
        BootTargetKind::AppleMacOsReboot => ChainloadDecision::RebootToApple,
        BootTargetKind::FirmwareSetup => ChainloadDecision::Reject {
            reason: "Apple Silicon has no UEFI firmware setup application",
        },
    }
}

/// Linux on this bridge needs the FDT U-Boot forwarded. ACPI-only images do not.
pub fn linux_requires_apple_device_tree(caps: &PlatformCapabilities) -> bool {
    caps.firmware == FirmwarePlatform::AppleSiliconM1n1Uboot
}

/// UEFI `Simple File System` on this bridge is the FAT ESP. APFS is not mounted.
pub fn apfs_visible_to_uefi() -> bool {
    false
}

fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let hyphen = matches!(index, 8 | 13 | 18 | 23);
        if hyphen && *byte != b'-' {
            return false;
        }
        if !hyphen && !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{SocId, soc_profile};
    use sbm_types::OsFamily;

    #[test]
    fn macos_target_is_a_reboot_id_and_not_an_efi_path() {
        let target = macos_reboot_target(
            "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
            "Macintosh HD",
            false,
        )
        .unwrap();
        assert_eq!(
            target.id.as_str(),
            "apple-macos:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
        assert!(target.id.is_valid());
        assert_eq!(target.kind, BootTargetKind::AppleMacOsReboot);
        assert!(target.efi_path.is_none());
        assert!(macos_reboot_target("not-a-uuid", "x", false).is_err());
    }

    #[test]
    fn apple_bridge_reboots_to_macos_and_can_start_linux_efi() {
        let caps = soc_profile(SocId::T8103).capabilities_if_bridge_is_up();
        let macos = macos_reboot_target(
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            "Macintosh HD",
            false,
        )
        .unwrap();
        assert_eq!(
            chainload_decision(&caps, &macos),
            ChainloadDecision::RebootToApple
        );
        let linux = BootTarget {
            id: BootTargetId("efi:example:abcdef012345".into()),
            display_name: "Linux".into(),
            os_family: OsFamily::Linux,
            kind: BootTargetKind::EfiBinary,
            arch: CpuArch::Aarch64,
            partition_guid: None,
            efi_path: Some("\\EFI\\Linux\\grubaa64.efi".into()),
            kernel_cmdline: None,
            icon_hint: None,
            removable_media: true,
            hidden: false,
            verified_signature: None,
        };
        assert_eq!(
            chainload_decision(&caps, &linux),
            ChainloadDecision::AllowEfi
        );
        assert!(linux_requires_apple_device_tree(&caps));
        assert!(!apfs_visible_to_uefi());
    }

    #[test]
    fn windows_and_foreign_architectures_are_rejected() {
        let caps = soc_profile(SocId::T8142).capabilities_if_bridge_is_up();
        let windows = BootTarget {
            id: BootTargetId("efi:example:abcdef012345".into()),
            display_name: "Windows".into(),
            os_family: OsFamily::Windows,
            kind: BootTargetKind::EfiBinary,
            arch: CpuArch::Aarch64,
            partition_guid: None,
            efi_path: Some("\\EFI\\Boot\\bootaa64.efi".into()),
            kernel_cmdline: None,
            icon_hint: None,
            removable_media: true,
            hidden: false,
            verified_signature: None,
        };
        assert!(matches!(
            chainload_decision(&caps, &windows),
            ChainloadDecision::Reject { .. }
        ));
        let x86 = BootTarget {
            arch: CpuArch::X86_64,
            os_family: OsFamily::Linux,
            ..windows
        };
        assert!(matches!(
            chainload_decision(&caps, &x86),
            ChainloadDecision::Reject { .. }
        ));
    }
}
