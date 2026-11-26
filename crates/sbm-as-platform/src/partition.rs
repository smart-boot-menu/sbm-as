//! Partition protection for Apple Silicon disks.
//!
//! GPT type GUIDs are the stable identifiers. `diskutil` content strings are
//! accepted because that is what a read-only macOS probe reports. APFS roles
//! are the volume roles, not display labels.

use sbm_types::PartitionProtectionClass;

/// GPT type for the iBoot System Container (`Apple_APFS_ISC`).
pub const GPT_IBOOT_SYSTEM_CONTAINER: &str = "69646961-6700-11AA-AA11-00306543ECAC";

/// GPT type for a normal APFS container (`Apple_APFS`).
pub const GPT_APPLE_APFS: &str = "7C3457EF-0000-11AA-AA11-00306543ECAC";

/// GPT type for System Recovery (`Apple_APFS_Recovery`).
pub const GPT_APPLE_RECOVERY: &str = "52637672-7900-11AA-AA11-00306543ECAC";

/// UEFI EFI System Partition type.
pub const GPT_EFI_SYSTEM: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";

/// What a caller observed about one GPT partition or one APFS volume.
#[derive(Debug, Clone, Copy, Default)]
pub struct PartitionView<'a> {
    pub gpt_type_guid: Option<&'a str>,
    pub gpt_name: Option<&'a str>,
    pub content: Option<&'a str>,
    pub apfs_role: Option<&'a str>,
    pub volume_name: Option<&'a str>,
    pub hosts_bootloader: bool,
    pub is_system_esp: bool,
    pub is_sbm_recovery: bool,
    pub encrypted: bool,
}

pub fn classify(view: PartitionView<'_>) -> PartitionProtectionClass {
    if is_apple_platform(view) {
        return PartitionProtectionClass::ProtectedAppleSystem;
    }
    if view.is_sbm_recovery {
        return PartitionProtectionClass::ProtectedSbmRecovery;
    }
    if view.encrypted {
        return PartitionProtectionClass::ProtectedEncrypted;
    }
    if view.is_system_esp || (view.hosts_bootloader && is_efi(view)) {
        return PartitionProtectionClass::ProtectedEsp;
    }
    PartitionProtectionClass::Normal
}

fn is_apple_platform(view: PartitionView<'_>) -> bool {
    guid_is(view.gpt_type_guid, GPT_IBOOT_SYSTEM_CONTAINER)
        || guid_is(view.gpt_type_guid, GPT_APPLE_APFS)
        || guid_is(view.gpt_type_guid, GPT_APPLE_RECOVERY)
        || content_is(view.content, "Apple_APFS_ISC")
        || content_is(view.content, "Apple_APFS")
        || content_is(view.content, "Apple_APFS_Recovery")
        || name_is(view.gpt_name, "iBootSystemContainer")
        || name_is(view.gpt_name, "RecoveryOSContainer")
        || name_is(view.volume_name, "iSCPreboot")
        || apple_role(view.apfs_role)
}

fn apple_role(role: Option<&str>) -> bool {
    let Some(role) = role else {
        return false;
    };
    const ROLES: &[&str] = &[
        "system", "data", "preboot", "recovery", "vm", "update", "xart", "hardware", "prelogin",
    ];
    let role = role.trim();
    ROLES
        .iter()
        .any(|candidate| role.eq_ignore_ascii_case(candidate))
}

fn is_efi(view: PartitionView<'_>) -> bool {
    guid_is(view.gpt_type_guid, GPT_EFI_SYSTEM) || content_is(view.content, "EFI")
}

fn guid_is(value: Option<&str>, expected: &str) -> bool {
    match value {
        Some(value) => value.eq_ignore_ascii_case(expected),
        None => false,
    }
}

fn content_is(value: Option<&str>, expected: &str) -> bool {
    match value {
        Some(value) => value.eq_ignore_ascii_case(expected),
        None => false,
    }
}

fn name_is(value: Option<&str>, expected: &str) -> bool {
    match value {
        Some(value) => value.eq_ignore_ascii_case(expected),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbm_types::{
        DiskBus, DiskIdentity, PartitionProtectionClass, SCHEMA_VERSION_V1, StorageOperationPlan,
    };

    fn view() -> PartitionView<'static> {
        PartitionView::default()
    }

    #[test]
    fn apple_containers_and_volumes_are_blocked() {
        let cases = [
            PartitionView {
                content: Some("Apple_APFS_ISC"),
                ..view()
            },
            PartitionView {
                gpt_type_guid: Some(GPT_APPLE_RECOVERY),
                ..view()
            },
            PartitionView {
                gpt_name: Some("iBootSystemContainer"),
                ..view()
            },
            PartitionView {
                volume_name: Some("iSCPreboot"),
                ..view()
            },
            PartitionView {
                apfs_role: Some("Data"),
                ..view()
            },
            PartitionView {
                apfs_role: Some("Preboot"),
                ..view()
            },
            PartitionView {
                content: Some("Apple_APFS"),
                ..view()
            },
        ];
        for case in cases {
            assert_eq!(
                classify(case),
                PartitionProtectionClass::ProtectedAppleSystem
            );
        }
    }

    #[test]
    fn blank_esp_can_be_formatted_and_system_esp_cannot() {
        let blank = PartitionView {
            gpt_type_guid: Some(GPT_EFI_SYSTEM),
            ..view()
        };
        assert_eq!(classify(blank), PartitionProtectionClass::Normal);
        let system = PartitionView {
            content: Some("EFI"),
            is_system_esp: true,
            ..view()
        };
        assert_eq!(classify(system), PartitionProtectionClass::ProtectedEsp);
    }

    #[test]
    fn linux_and_sbm_recovery_are_distinct() {
        let linux = PartitionView {
            gpt_type_guid: Some("B921B045-1DF0-41C3-AF44-4C6F280D3FAE"),
            ..view()
        };
        assert_eq!(classify(linux), PartitionProtectionClass::Normal);
        let recovery = PartitionView {
            is_sbm_recovery: true,
            ..view()
        };
        assert_eq!(
            classify(recovery),
            PartitionProtectionClass::ProtectedSbmRecovery
        );
        let luks = PartitionView {
            encrypted: true,
            ..view()
        };
        assert_eq!(classify(luks), PartitionProtectionClass::ProtectedEncrypted);
    }

    #[test]
    fn classified_apple_partition_blocks_a_storage_plan() {
        let class = classify(PartitionView {
            content: Some("Apple_APFS_Recovery"),
            ..view()
        });
        let plan = StorageOperationPlan {
            schema_version: SCHEMA_VERSION_V1.into(),
            plan_id: "019a0000-0000-7000-8000-000000000010".into(),
            target_disk: DiskIdentity {
                stable_id: "gpt:example".into(),
                gpt_disk_guid: None,
                wwn_or_eui64: None,
                model: "fixture".into(),
                serial_hash: None,
                size_bytes: 0,
                logical_sector_size: 4096,
                bus: DiskBus::Nvme,
                removable: false,
                read_only: true,
            },
            expected_partition_table_sha256: "00".into(),
            operations: alloc::vec![],
            destructive: true,
            reversible: false,
            touched_protected_classes: alloc::vec![class],
            dry_run_summary: alloc::vec!["blocked".into()],
        };
        assert!(plan.blocks_by_default_safety());
    }
}
