#![no_std]
#![forbid(unsafe_code)]

//! Bounded, evidence-aware hardware profile assembled from trusted probes.
//! Unknown is distinct from a measured lack of support.

use boot_protocol::{graphics::FramebufferInfo, MemoryMap, PAGE_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation<T> {
    Known(T),
    Unsupported,
    Unknown,
}

impl<T> Observation<T> {
    pub const fn unknown() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: PixelFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderCapabilities {
    pub framebuffer: Observation<DisplayMode>,
    pub gpu_acceleration: Observation<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputeCapabilities {
    pub logical_processors: Observation<u16>,
    pub vector_extensions: Observation<u128>,
    pub gpu_compute: Observation<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryCapabilities {
    /// Bytes in UEFI ConventionalMemory, not total installed or allocatable RAM.
    pub conventional_bytes: Observation<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageInterface {
    Hdd,
    Sata,
    Nvme,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageCapabilities {
    pub device_count: Observation<u16>,
    pub total_bytes: Observation<u64>,
    pub fastest_interface: Observation<StorageInterface>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayCapabilities {
    pub mode: Observation<DisplayMode>,
    pub refresh_rate_millihz: Observation<u32>,
    pub hdr: Observation<bool>,
    pub multi_monitor: Observation<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaCapabilities {
    pub hardware_decode: Observation<bool>,
    pub hardware_encode: Observation<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiAccelerator {
    Gpu,
    Npu,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiCapabilities {
    pub accelerator: Observation<AiAccelerator>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerSource {
    Battery,
    Ac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerCapabilities {
    pub source: Observation<PowerSource>,
    pub low_power_mode: Observation<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareProfile {
    pub render: RenderCapabilities,
    pub compute: ComputeCapabilities,
    pub memory: MemoryCapabilities,
    pub storage: StorageCapabilities,
    pub display: DisplayCapabilities,
    pub media: MediaCapabilities,
    pub ai: AiCapabilities,
    pub power: PowerCapabilities,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    InvalidFramebuffer,
    MemoryTotalOverflow,
}

impl HardwareProfile {
    /// Builds a partial profile from boot data that has already passed the
    /// boot protocol's memory-map and framebuffer validation.
    pub fn from_boot(
        memory_map: &MemoryMap<'_>,
        framebuffer: Option<FramebufferInfo>,
    ) -> Result<Self, ProfileError> {
        let mut conventional_bytes = 0u64;
        for index in 0..memory_map.len() {
            let descriptor = memory_map
                .descriptor(index)
                .expect("validated memory map descriptor");
            if descriptor.memory_type == 7 {
                let bytes = descriptor
                    .number_of_pages
                    .checked_mul(PAGE_SIZE)
                    .ok_or(ProfileError::MemoryTotalOverflow)?;
                conventional_bytes = conventional_bytes
                    .checked_add(bytes)
                    .ok_or(ProfileError::MemoryTotalOverflow)?;
            }
        }

        let mode = match framebuffer {
            Some(framebuffer) => {
                framebuffer
                    .validate()
                    .map_err(|_| ProfileError::InvalidFramebuffer)?;
                Observation::Known(DisplayMode {
                    width: framebuffer.width,
                    height: framebuffer.height,
                    stride: framebuffer.stride,
                    pixel_format: if framebuffer.pixel_format == 0 {
                        PixelFormat::Rgb
                    } else {
                        PixelFormat::Bgr
                    },
                })
            }
            None => Observation::Unknown,
        };

        Ok(Self {
            render: RenderCapabilities {
                framebuffer: mode,
                gpu_acceleration: Observation::Unknown,
            },
            compute: ComputeCapabilities {
                logical_processors: Observation::Unknown,
                vector_extensions: Observation::Unknown,
                gpu_compute: Observation::Unknown,
            },
            memory: MemoryCapabilities {
                conventional_bytes: Observation::Known(conventional_bytes),
            },
            storage: StorageCapabilities {
                device_count: Observation::Unknown,
                total_bytes: Observation::Unknown,
                fastest_interface: Observation::Unknown,
            },
            display: DisplayCapabilities {
                mode,
                refresh_rate_millihz: Observation::Unknown,
                hdr: Observation::Unknown,
                multi_monitor: Observation::Unknown,
            },
            media: MediaCapabilities {
                hardware_decode: Observation::Unknown,
                hardware_encode: Observation::Unknown,
            },
            ai: AiCapabilities {
                accelerator: Observation::Unknown,
            },
            power: PowerCapabilities {
                source: Observation::Unknown,
                low_power_mode: Observation::Unknown,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(memory_type: u32, start: u64, pages: u64) -> [u8; 40] {
        let mut bytes = [0; 40];
        bytes[..4].copy_from_slice(&memory_type.to_le_bytes());
        bytes[8..16].copy_from_slice(&start.to_le_bytes());
        bytes[24..32].copy_from_slice(&pages.to_le_bytes());
        bytes
    }

    fn memory_map(bytes: &mut [u8; 80]) -> MemoryMap<'_> {
        *bytes = [0; 80];
        bytes[..40].copy_from_slice(&descriptor(7, 0x1000, 256));
        bytes[40..].copy_from_slice(&descriptor(2, 0x101000, 32));
        MemoryMap::new(bytes, 40, 1).unwrap()
    }

    fn framebuffer() -> FramebufferInfo {
        FramebufferInfo {
            magic: boot_protocol::graphics::FRAMEBUFFER_MAGIC,
            base: 0x8000_0000,
            byte_len: 4 * 1024 * 1024,
            width: 1280,
            height: 720,
            stride: 1280,
            pixel_format: 1,
            reserved: [0; 3],
        }
    }

    #[test]
    fn boot_profile_reports_only_observed_memory_and_display_facts() {
        let mut bytes = [0; 80];
        let map = memory_map(&mut bytes);
        let profile = HardwareProfile::from_boot(&map, Some(framebuffer())).unwrap();
        assert_eq!(
            profile.memory.conventional_bytes,
            Observation::Known(256 * PAGE_SIZE)
        );
        let expected = DisplayMode {
            width: 1280,
            height: 720,
            stride: 1280,
            pixel_format: PixelFormat::Bgr,
        };
        assert_eq!(profile.render.framebuffer, Observation::Known(expected));
        assert_eq!(profile.display.mode, Observation::Known(expected));
        assert_eq!(profile.render.gpu_acceleration, Observation::Unknown);
        assert_eq!(profile.compute.logical_processors, Observation::Unknown);
        assert_eq!(profile.storage.total_bytes, Observation::Unknown);
        assert_eq!(profile.media.hardware_decode, Observation::Unknown);
        assert_eq!(profile.ai.accelerator, Observation::Unknown);
        assert_eq!(profile.power.source, Observation::Unknown);
    }

    #[test]
    fn missing_framebuffer_is_not_reported_as_unsupported() {
        let mut bytes = [0; 80];
        let map = memory_map(&mut bytes);
        let profile = HardwareProfile::from_boot(&map, None).unwrap();
        assert_eq!(profile.render.framebuffer, Observation::Unknown);
        assert_eq!(profile.display.mode, Observation::Unknown);
    }

    #[test]
    fn malformed_framebuffer_is_rejected() {
        let mut bytes = [0; 80];
        let map = memory_map(&mut bytes);
        let mut invalid = framebuffer();
        invalid.width = 0;
        assert_eq!(
            HardwareProfile::from_boot(&map, Some(invalid)),
            Err(ProfileError::InvalidFramebuffer)
        );
    }

    #[test]
    fn only_conventional_memory_contributes_to_reported_memory() {
        let mut bytes = [0; 80];
        bytes[..40].copy_from_slice(&descriptor(7, 0x1000, 256));
        bytes[40..].copy_from_slice(&descriptor(2, 0x101000, 32));
        let map = MemoryMap::new(&bytes, 40, 1).unwrap();
        let profile = HardwareProfile::from_boot(&map, None).unwrap();
        assert_eq!(
            profile.memory.conventional_bytes,
            Observation::Known(256 * PAGE_SIZE)
        );
    }
}
