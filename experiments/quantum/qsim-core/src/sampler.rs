/// Deterministic simulation only. Never use for cryptographic keys or entropy.
/// Algorithm: Sebastiano Vigna (2015), https://prng.di.unimi.it/splitmix64.c
/// Upstream dedicates the algorithm implementation to the public domain (CC0).
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
    pub fn unit(&mut self) -> f64 {
        (self.word() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn published_seed_zero_words_and_exclusive_upper_bound() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.word(), 0xe220a8397b1dcdaf);
        assert_eq!(rng.word(), 0x6e789e6aa1b965f4);
        for _ in 0..10_000 {
            assert!((0.0..1.0).contains(&rng.unit()));
        }
    }
}
