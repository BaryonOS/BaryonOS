//! SP 800-90B §4.4 continuous health tests over 8-bit samples, with the
//! cutoffs for a conservatively assumed H = 4 bits per sample and
//! α = 2⁻²⁰: Repetition Count C = 1 + ⌈20 / 4⌉ = 6 and Adaptive Proportion
//! over a window of 512 with cutoff 62 (smallest c with
//! P[Bin(512, 2⁻⁴) ≥ c] ≤ 2⁻²⁰). The tests detect a stuck or heavily biased
//! source; they do not measure the true min-entropy of real hardware.

pub const RCT_CUTOFF: u32 = 6;
pub const APT_WINDOW: u32 = 512;
pub const APT_CUTOFF: u32 = 62;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HealthFailure {
    None = 0,
    /// The broker reported no device, a timeout or a short answer.
    Source = 1,
    /// `RCT_CUTOFF` identical consecutive samples.
    Repetition = 2,
    /// One value appeared `APT_CUTOFF` times within a window.
    Proportion = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct Health {
    last: Option<u8>,
    run: u32,
    window_sample: Option<u8>,
    window_count: u32,
    window_seen: u32,
    pub samples: u64,
}

impl Default for Health {
    fn default() -> Self {
        Self::new()
    }
}

impl Health {
    pub const fn new() -> Self {
        Self {
            last: None,
            run: 0,
            window_sample: None,
            window_count: 0,
            window_seen: 0,
            samples: 0,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), HealthFailure> {
        for &sample in bytes {
            self.samples += 1;
            // Repetition Count Test.
            if self.last == Some(sample) {
                self.run += 1;
                if self.run >= RCT_CUTOFF {
                    return Err(HealthFailure::Repetition);
                }
            } else {
                self.last = Some(sample);
                self.run = 1;
            }
            // Adaptive Proportion Test.
            match self.window_sample {
                None => {
                    self.window_sample = Some(sample);
                    self.window_count = 1;
                    self.window_seen = 1;
                }
                Some(first) => {
                    self.window_seen += 1;
                    if sample == first {
                        self.window_count += 1;
                        if self.window_count >= APT_CUTOFF {
                            return Err(HealthFailure::Proportion);
                        }
                    }
                    if self.window_seen >= APT_WINDOW {
                        self.window_sample = None;
                    }
                }
            }
        }
        Ok(())
    }
}
