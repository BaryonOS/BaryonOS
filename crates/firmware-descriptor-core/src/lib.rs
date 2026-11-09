#![no_std]

use core::num::{NonZeroU32, NonZeroU64};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareId(NonZeroU64);

impl FirmwareId {
    pub const fn new(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceMatch {
    vendor_id: u16,
    device_id: u16,
}

impl DeviceMatch {
    pub const fn new(vendor_id: u16, device_id: u16) -> Option<Self> {
        if vendor_id == 0 || vendor_id == u16::MAX {
            None
        } else {
            Some(Self {
                vendor_id,
                device_id,
            })
        }
    }

    pub const fn vendor_id(self) -> u16 {
        self.vendor_id
    }

    pub const fn device_id(self) -> u16 {
        self.device_id
    }

    pub const fn matches(self, vendor_id: u16, device_id: u16) -> bool {
        self.vendor_id == vendor_id && self.device_id == device_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareHash([u8; 32]);

impl FirmwareHash {
    pub const fn new(value: [u8; 32]) -> Self {
        Self(value)
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LicenseReference(NonZeroU32);

impl LicenseReference {
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    BuiltInPackage(NonZeroU32),
    VendorManifest(NonZeroU32),
    UserProvided,
    DeveloperBuild,
}

impl Provenance {
    pub const fn built_in(package: u32) -> Option<Self> {
        match NonZeroU32::new(package) {
            Some(package) => Some(Self::BuiltInPackage(package)),
            None => None,
        }
    }

    pub const fn vendor_manifest(reference: u32) -> Option<Self> {
        match NonZeroU32::new(reference) {
            Some(reference) => Some(Self::VendorManifest(reference)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompatibilityRequirements {
    pub minimum_device_revision: u16,
    pub maximum_device_revision: u16,
    pub required_platform_features: u64,
}

impl CompatibilityRequirements {
    pub const fn validate(self) -> Result<Self, Error> {
        if self.minimum_device_revision > self.maximum_device_revision {
            Err(Error::InvalidCompatibilityRange)
        } else {
            Ok(self)
        }
    }

    pub const fn matches(self, device_revision: u16, platform_features: u64) -> bool {
        device_revision >= self.minimum_device_revision
            && device_revision <= self.maximum_device_revision
            && platform_features & self.required_platform_features
                == self.required_platform_features
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmwareDescriptor {
    pub id: FirmwareId,
    pub device: DeviceMatch,
    pub version: Version,
    pub digest: FirmwareHash,
    pub provenance: Provenance,
    pub license: LicenseReference,
    pub compatibility: CompatibilityRequirements,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedDescriptor {
    descriptor: FirmwareDescriptor,
}

impl ValidatedDescriptor {
    pub const fn descriptor(self) -> FirmwareDescriptor {
        self.descriptor
    }

    pub const fn verification_status(self) -> VerificationStatus {
        VerificationStatus::Unverified
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidCompatibilityRange,
}

impl FirmwareDescriptor {
    pub const fn validate(self) -> Result<ValidatedDescriptor, Error> {
        match self.compatibility.validate() {
            Ok(_) => Ok(ValidatedDescriptor { descriptor: self }),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor() -> FirmwareDescriptor {
        FirmwareDescriptor {
            id: FirmwareId::new(1).unwrap(),
            device: DeviceMatch::new(0x1234, 0x5678).unwrap(),
            version: Version {
                major: 2,
                minor: 1,
                patch: 0,
            },
            digest: FirmwareHash::new([0xab; 32]),
            provenance: Provenance::built_in(7).unwrap(),
            license: LicenseReference::new(5).unwrap(),
            compatibility: CompatibilityRequirements {
                minimum_device_revision: 2,
                maximum_device_revision: 8,
                required_platform_features: 0b0101,
            },
        }
    }

    #[test]
    fn rejects_empty_or_invalid_descriptor_fields() {
        assert!(FirmwareId::new(0).is_none());
        assert!(DeviceMatch::new(0, 1).is_none());
        assert!(DeviceMatch::new(u16::MAX, 1).is_none());
        assert!(DeviceMatch::new(1, 0).is_some());
        assert_eq!(FirmwareHash::new([0; 32]).bytes(), &[0; 32]);
        assert!(LicenseReference::new(0).is_none());
        assert!(Provenance::built_in(0).is_none());
        assert!(Provenance::vendor_manifest(0).is_none());
    }

    #[test]
    fn validates_metadata_without_promoting_trust() {
        let validated = descriptor().validate().unwrap();
        assert_eq!(validated.descriptor(), descriptor());
        assert_eq!(
            validated.verification_status(),
            VerificationStatus::Unverified
        );
    }

    #[test]
    fn exact_device_match_and_compatibility_requirements_are_enforced() {
        let descriptor = descriptor().validate().unwrap().descriptor();
        assert!(descriptor.device.matches(0x1234, 0x5678));
        assert!(!descriptor.device.matches(0x1234, 0x5679));
        assert!(descriptor.compatibility.matches(2, 0b0101));
        assert!(descriptor.compatibility.matches(8, 0b1111));
        assert!(!descriptor.compatibility.matches(1, 0b1111));
        assert!(!descriptor.compatibility.matches(9, 0b1111));
        assert!(!descriptor.compatibility.matches(4, 0b0001));
    }

    #[test]
    fn malformed_compatibility_range_fails_validation() {
        let mut descriptor = descriptor();
        descriptor.compatibility.minimum_device_revision = 9;
        descriptor.compatibility.maximum_device_revision = 8;
        assert_eq!(descriptor.validate(), Err(Error::InvalidCompatibilityRange));
    }

    #[test]
    fn version_is_a_typed_numeric_tuple() {
        assert!(
            Version {
                major: 1,
                minor: 10,
                patch: 0,
            } > Version {
                major: 1,
                minor: 2,
                patch: 9,
            }
        );
    }
}
