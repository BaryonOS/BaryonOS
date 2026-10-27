/// Targets remain wide until validation; no narrowing of untrusted integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    H { target: u64 },
    X { target: u64 },
    Z { target: u64 },
    S { target: u64 },
    Cx { control: u64, target: u64 },
}

impl Gate {
    pub fn target(self) -> u64 {
        match self {
            Self::H { target }
            | Self::X { target }
            | Self::Z { target }
            | Self::S { target }
            | Self::Cx { target, .. } => target,
        }
    }
}
