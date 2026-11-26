//! External-boot feasibility, split by generation.
//!
//! Levels describe the platform and upstream software. They do not mean SBM
//! has booted that scenario on hardware.

use crate::profile::AppleGeneration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalScenario {
    AppleExternalMacOs,
    ExternalArm64Linux,
    ExternalRootInternalBoot,
    GenericArm64UefiImage,
    OtherArm64OperatingSystem,
    X86ThroughEmulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    SupportedToday,
    DemonstratedExperimentally,
    RequiresUpstream,
    TheoreticallyPossible,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationLevels {
    pub m1: SupportLevel,
    pub m2: SupportLevel,
    pub m3: SupportLevel,
    pub m3_ultra: SupportLevel,
    pub m4: SupportLevel,
    pub m5: SupportLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioAssessment {
    pub scenario: ExternalScenario,
    pub levels: GenerationLevels,
    /// What SBM can add while leaving Apple's boot policy in place.
    pub ux_within_apple_security: &'static str,
}

pub fn assessments() -> &'static [ScenarioAssessment] {
    use ExternalScenario as S;
    use SupportLevel as L;
    const ROWS: &[ScenarioAssessment] = &[
        ScenarioAssessment {
            scenario: S::AppleExternalMacOs,
            levels: GenerationLevels {
                m1: L::SupportedToday,
                m2: L::SupportedToday,
                m3: L::SupportedToday,
                m3_ultra: L::SupportedToday,
                m4: L::SupportedToday,
                m5: L::SupportedToday,
            },
            ux_within_apple_security: "Show the volume as an Apple reboot target. Selection returns to iBoot through the startup picker. SBM does not write LocalPolicy and does not start boot.efi.",
        },
        ScenarioAssessment {
            scenario: S::ExternalArm64Linux,
            levels: GenerationLevels {
                m1: L::DemonstratedExperimentally,
                m2: L::DemonstratedExperimentally,
                m3: L::DemonstratedExperimentally,
                m3_ultra: L::RequiresUpstream,
                m4: L::RequiresUpstream,
                m5: L::RequiresUpstream,
            },
            ux_within_apple_security: "Once U-Boot has published USB block devices, discover FAT ESPs on them. The pinned Asahi defconfig does not start USB before bootflow, so the bootstrap fragment has to run usb start first.",
        },
        ScenarioAssessment {
            scenario: S::ExternalRootInternalBoot,
            levels: GenerationLevels {
                m1: L::SupportedToday,
                m2: L::SupportedToday,
                m3: L::SupportedToday,
                m3_ultra: L::RequiresUpstream,
                m4: L::RequiresUpstream,
                m5: L::RequiresUpstream,
            },
            ux_within_apple_security: "Keep m1n1 stage 2, U-Boot, and the device tree on the internal System ESP, and let one owner update them. The OS root can live on external storage. This is the Asahi-documented layout, not a bypass.",
        },
        ScenarioAssessment {
            scenario: S::GenericArm64UefiImage,
            levels: GenerationLevels {
                m1: L::TheoreticallyPossible,
                m2: L::TheoreticallyPossible,
                m3: L::TheoreticallyPossible,
                m3_ultra: L::RequiresUpstream,
                m4: L::RequiresUpstream,
                m5: L::RequiresUpstream,
            },
            ux_within_apple_security: "U-Boot can StartImage a PE/COFF aarch64 binary. A generic image that needs ACPI, 4 KiB pages, or non-Apple drivers does not become a running OS. Warn before offering it.",
        },
        ScenarioAssessment {
            scenario: S::OtherArm64OperatingSystem,
            levels: GenerationLevels {
                m1: L::SupportedToday,
                m2: L::SupportedToday,
                m3: L::Unsupported,
                m3_ultra: L::Unsupported,
                m4: L::Unsupported,
                m5: L::Unsupported,
            },
            ux_within_apple_security: "OpenBSD/arm64 lists Apple M1 and M2, requires the Asahi installer, and then boots its own installer from USB. That page does not list M3, M4, or M5. SBM has not booted OpenBSD.",
        },
        ScenarioAssessment {
            scenario: S::X86ThroughEmulation,
            levels: GenerationLevels {
                m1: L::SupportedToday,
                m2: L::SupportedToday,
                m3: L::SupportedToday,
                m3_ultra: L::SupportedToday,
                m4: L::SupportedToday,
                m5: L::SupportedToday,
            },
            ux_within_apple_security: "x86 systems run as guests of a host OS that is already booted. SBM rejects them as native EFI chainload targets.",
        },
    ];
    ROWS
}

pub fn level_for(
    levels: GenerationLevels,
    generation: AppleGeneration,
    ultra: bool,
) -> SupportLevel {
    if ultra && generation == AppleGeneration::M3 {
        return levels.m3_ultra;
    }
    match generation {
        AppleGeneration::M1 => levels.m1,
        AppleGeneration::M2 => levels.m2,
        AppleGeneration::M3 => levels.m3,
        AppleGeneration::M4 => levels.m4,
        AppleGeneration::M5 => levels.m5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BYPASSES_APPLE_BOOT_SECURITY;

    #[test]
    fn no_scenario_is_a_security_bypass_and_m5_linux_is_upstream_work() {
        const { assert!(!BYPASSES_APPLE_BOOT_SECURITY) };
        assert_eq!(assessments().len(), 6);
        for row in assessments() {
            assert!(!row.ux_within_apple_security.is_empty());
        }
        let linux = assessments()
            .iter()
            .find(|row| row.scenario == ExternalScenario::ExternalArm64Linux)
            .unwrap();
        assert_eq!(
            level_for(linux.levels, AppleGeneration::M5, false),
            SupportLevel::RequiresUpstream
        );
        assert_eq!(
            level_for(linux.levels, AppleGeneration::M1, false),
            SupportLevel::DemonstratedExperimentally
        );
        let generic = assessments()
            .iter()
            .find(|row| row.scenario == ExternalScenario::GenericArm64UefiImage)
            .unwrap();
        assert_eq!(generic.levels.m1, SupportLevel::TheoreticallyPossible);
        assert_ne!(generic.levels.m1, SupportLevel::SupportedToday);
    }
}
