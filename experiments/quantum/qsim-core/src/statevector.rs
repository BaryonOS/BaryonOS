use crate::{complex::Complex, Control, ExecuteError, Gate, ValidatedJob};
use alloc::vec::Vec;

pub(crate) fn evolve(
    job: &ValidatedJob,
    control: &mut impl Control,
) -> Result<(Vec<Complex>, u64), ExecuteError> {
    control.checkpoint(0)?;
    let len = 1_usize << job.qubits();
    let mut state = Vec::new();
    state
        .try_reserve_exact(len)
        .map_err(|_| ExecuteError::ResourceExhausted)?;
    let mut work = 0;
    for index in 0..len {
        if index % 256 == 0 {
            control.checkpoint(work)?;
        }
        state.push(if index == 0 {
            Complex::ONE
        } else {
            Complex::ZERO
        });
        work += 1;
    }
    for &gate in job.gates() {
        control.checkpoint(work)?;
        apply(&mut state, gate, control, &mut work)?;
    }
    Ok((state, work))
}

fn apply(
    state: &mut [Complex],
    gate: Gate,
    control: &mut impl Control,
    work: &mut u64,
) -> Result<(), ExecuteError> {
    let bit = 1_usize << gate.target();
    for i in 0..state.len() {
        if i % 256 == 0 {
            control.checkpoint(*work)?;
        }
        *work += 1;
        if i & bit != 0 {
            continue;
        }
        let j = i | bit;
        let a = state[i];
        let b = state[j];
        match gate {
            Gate::X { .. } => state.swap(i, j),
            Gate::H { .. } => {
                let scale = core::f64::consts::FRAC_1_SQRT_2;
                state[i] = Complex {
                    re: (a.re + b.re) * scale,
                    im: (a.im + b.im) * scale,
                };
                state[j] = Complex {
                    re: (a.re - b.re) * scale,
                    im: (a.im - b.im) * scale,
                };
            }
            Gate::Z { .. } => {
                state[j] = Complex {
                    re: -b.re,
                    im: -b.im,
                }
            }
            Gate::S { .. } => {
                state[j] = Complex {
                    re: -b.im,
                    im: b.re,
                }
            }
            Gate::Cx { control, .. } => {
                if i & (1_usize << control) != 0 {
                    state.swap(i, j);
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn distribution(
    state: &[Complex],
    control: &mut impl Control,
    work: &mut u64,
) -> Result<Vec<f64>, ExecuteError> {
    control.checkpoint(*work)?;
    let mut cdf = Vec::new();
    cdf.try_reserve_exact(state.len())
        .map_err(|_| ExecuteError::ResourceExhausted)?;
    let mut sum = 0.0;
    for (index, &amplitude) in state.iter().enumerate() {
        if index % 256 == 0 {
            control.checkpoint(*work)?;
        }
        *work += 1;
        let probability = amplitude.probability();
        if !probability.is_finite() || probability < 0.0 {
            return Err(ExecuteError::NumericalError);
        }
        sum += probability;
        cdf.push(probability);
    }
    if !sum.is_finite() || (sum - 1.0).abs() > 1e-9 {
        return Err(ExecuteError::NumericalError);
    }
    let mut cumulative = 0.0;
    for (index, probability) in cdf.iter_mut().enumerate() {
        if index % 256 == 0 {
            control.checkpoint(*work)?;
        }
        *work += 1;
        cumulative += *probability / sum;
        // Bound cumulative rounding to keep trailing zero-mass bins monotone
        // when the final CDF entry is set to exactly one.
        *probability = cumulative.min(1.0);
    }
    *cdf.last_mut().ok_or(ExecuteError::NumericalError)? = 1.0;
    Ok(cdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uninterrupted;
    use alloc::vec;

    fn vector(gates: Vec<Gate>, n: u64) -> Vec<Complex> {
        evolve(
            &ValidatedJob::new(n, gates, 1, 0).unwrap(),
            &mut Uninterrupted,
        )
        .unwrap()
        .0
    }
    fn close(actual: &[Complex], expected: &[Complex]) {
        assert_eq!(actual.len(), expected.len());
        for (a, b) in actual.iter().zip(expected) {
            assert!((a.re - b.re).abs() <= 1e-12, "{a:?} != {b:?}");
            assert!((a.im - b.im).abs() <= 1e-12, "{a:?} != {b:?}");
        }
    }
    #[test]
    fn q1_03_h_twice_is_identity() {
        close(
            &vector(vec![Gate::H { target: 0 }; 2], 1),
            &[Complex::ONE, Complex::ZERO],
        );
    }
    #[test]
    fn q1_04_gate_cycles_restore_basis_and_superposition() {
        for prefix in [
            vec![],
            vec![Gate::X { target: 0 }],
            vec![Gate::H { target: 0 }, Gate::S { target: 0 }],
            vec![Gate::H { target: 1 }, Gate::X { target: 0 }],
        ] {
            let baseline = vector(prefix.clone(), 2);
            for (gate, repeats) in [
                (Gate::X { target: 0 }, 2),
                (Gate::Z { target: 0 }, 2),
                (Gate::S { target: 0 }, 4),
                (
                    Gate::Cx {
                        control: 0,
                        target: 1,
                    },
                    2,
                ),
            ] {
                let mut gates = prefix.clone();
                gates.extend(core::iter::repeat_n(gate, repeats));
                close(&vector(gates, 2), &baseline);
            }
        }
    }
    #[test]
    fn q1_05_s_changes_phase() {
        let half = core::f64::consts::FRAC_1_SQRT_2;
        close(
            &vector(vec![Gate::H { target: 0 }, Gate::S { target: 0 }], 1),
            &[Complex { re: half, im: 0.0 }, Complex { re: 0.0, im: half }],
        );
    }
    #[test]
    fn q1_06_bell_probabilities_and_norm() {
        let state = vector(
            vec![
                Gate::H { target: 0 },
                Gate::Cx {
                    control: 0,
                    target: 1,
                },
            ],
            2,
        );
        for (a, expected) in state.iter().zip([0.5, 0.0, 0.0, 0.5]) {
            assert!((a.probability() - expected).abs() <= 1e-12);
        }
        assert!((state.iter().map(|a| a.probability()).sum::<f64>() - 1.0).abs() <= 1e-12);
    }
    #[test]
    fn q1_15_invalid_internal_states_are_rejected() {
        for invalid in [f64::NAN, f64::INFINITY, 2.0, 0.0] {
            assert_eq!(
                distribution(
                    &[Complex {
                        re: invalid,
                        im: 0.0
                    }],
                    &mut Uninterrupted,
                    &mut 0
                ),
                Err(ExecuteError::NumericalError)
            );
        }
        assert_eq!(
            distribution(&[], &mut Uninterrupted, &mut 0),
            Err(ExecuteError::NumericalError)
        );
    }
}
