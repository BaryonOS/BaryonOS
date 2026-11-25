use baryon_ai_reference_forward::{
    attention, kv_cache_bytes, rope, Budget, Cancel, Error, KvCache, NoCancel, MAX_KV_BYTES,
};

fn nb() -> Budget<'static> {
    Budget::new(u64::MAX, &NoCancel)
}

fn close(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
}

#[test]
fn kv_budget_uses_checked_formula_and_proposed_ceiling() {
    assert_eq!(kv_cache_bytes(2, 3, 4, 5, 4), Ok(2 * 2 * 3 * 4 * 5 * 4));
    assert_eq!(
        kv_cache_bytes(1, 1, 1, MAX_KV_BYTES / 2, 1),
        Ok(MAX_KV_BYTES)
    );
    assert_eq!(
        kv_cache_bytes(1, 1, 1, MAX_KV_BYTES / 2 + 1, 1),
        Err(Error::ContextLimit)
    );
    assert_eq!(kv_cache_bytes(usize::MAX, 2, 1, 1, 1), Err(Error::Shape));
    assert_eq!(kv_cache_bytes(1, 0, 1, 1, 1), Err(Error::Shape));
}

#[test]
fn rope_is_identity_at_zero_and_rotates_rotate_half_pairs() {
    let mut out = [9.; 4];
    rope(&[1., 2., 3., 4.], 0, 10_000., &mut out).unwrap();
    assert!(close(&out, &[1., 2., 3., 4.]));
    // Pair (0, 2) uses inverse frequency 1, so position 1 rotates by one radian.
    rope(&[1., 0., 0., 0.], 1, 10_000., &mut out).unwrap();
    assert!(close(&out, &[1f32.cos(), 0., 1f32.sin(), 0.]));
    // Pair (1, 3) uses theta^(-1/2) = 0.01 for theta 10,000.
    rope(&[0., 1., 0., 0.], 100, 10_000., &mut out).unwrap();
    assert!(close(&out, &[0., 1f32.cos(), 0., 1f32.sin()]));
}

#[test]
fn rope_rejects_bad_shapes_and_values_without_publishing() {
    let mut out = [7.; 3];
    assert_eq!(rope(&[1., 2., 3.], 1, 10., &mut out), Err(Error::Shape));
    let mut out = [7.; 2];
    assert_eq!(rope(&[1.; 4], 1, 10., &mut out), Err(Error::Shape));
    for theta in [f32::NAN, 1.0, 0.5] {
        assert_eq!(rope(&[1., 2.], 1, theta, &mut out), Err(Error::NonFinite));
    }
    assert_eq!(
        rope(&[f32::NAN, 2.], 1, 10., &mut out),
        Err(Error::NonFinite)
    );
    assert_eq!(
        rope(&[f32::MAX, f32::MAX], 1, 10., &mut out),
        Err(Error::NonFinite)
    );
    assert_eq!(out, [7.; 2]);
}

#[test]
fn cache_refuses_overflow_instead_of_evicting() {
    let (mut k, mut v) = ([0.; 4], [0.; 4]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    assert_eq!(cache.capacity(), 2);
    cache.append(&[1., 2.], &[3., 4.]).unwrap();
    cache.append(&[5., 6.], &[7., 8.]).unwrap();
    assert_eq!(cache.append(&[0., 0.], &[0., 0.]), Err(Error::ContextLimit));
    assert_eq!(cache.append(&[0.], &[0., 0.]), Err(Error::Shape));
    assert_eq!(cache.len(), 2);
    assert_eq!((k, v), ([1., 2., 5., 6.], [3., 4., 7., 8.]));
    let (mut k, mut v) = ([0.; 3], [0.; 3]);
    assert!(matches!(
        KvCache::new(1, 2, &mut k, &mut v),
        Err(Error::Shape)
    ));
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    assert_eq!(
        cache.append(&[f32::NAN, 0.], &[0., 0.]),
        Err(Error::NonFinite)
    );
    assert!(cache.is_empty());
}

#[test]
fn attention_matches_hand_computed_softmax() {
    let (mut k, mut v) = ([0.; 4], [0.; 4]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    cache.append(&[1., 0.], &[10., 0.]).unwrap();
    let mut out = [0.; 2];
    // A single token receives all weight.
    attention(&[3., 3.], 1, &cache, &mut [0.; 3], &mut out, &mut nb()).unwrap();
    assert!(close(&out, &[10., 0.]));
    cache.append(&[0., 1.], &[0., 20.]).unwrap();
    // Equal scores give the mean of the values.
    attention(&[1., 1.], 1, &cache, &mut [0.; 4], &mut out, &mut nb()).unwrap();
    assert!(close(&out, &[5., 10.]));
    // Scores 2/sqrt(2) and 0: weights e^s / (e^s + 1).
    attention(&[2., 0.], 1, &cache, &mut [0.; 4], &mut out, &mut nb()).unwrap();
    let w = 1.0 / (1.0 + (-(2f32 / 2f32.sqrt())).exp());
    assert!(close(&out, &[10. * w, 20. * (1. - w)]));
}

#[test]
fn grouped_query_heads_share_their_kv_head() {
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(2, 1, &mut k, &mut v).unwrap();
    cache.append(&[1., 1.], &[3., 7.]).unwrap();
    let mut out = [0.; 4];
    attention(
        &[1., 1., 1., 1.],
        4,
        &cache,
        &mut [0.; 5],
        &mut out,
        &mut nb(),
    )
    .unwrap();
    assert_eq!(out, [3., 3., 7., 7.]);
    assert_eq!(
        attention(&[1.; 3], 3, &cache, &mut [0.; 4], &mut [0.; 3], &mut nb()),
        Err(Error::Shape)
    );
}

#[test]
fn grouped_heads_read_their_own_kv_head_over_two_tokens() {
    // Two KV heads (head_dim 1), four query heads: heads 0-1 use KV head 0,
    // heads 2-3 use KV head 1. Keys and queries differ per head, so a wrong
    // head mapping changes the softmax weights.
    let (mut k, mut v) = ([0.; 4], [0.; 4]);
    let mut cache = KvCache::new(2, 1, &mut k, &mut v).unwrap();
    cache.append(&[1., -1.], &[10., 30.]).unwrap();
    cache.append(&[-1., 1.], &[20., 40.]).unwrap();
    let mut out = [0.; 4];
    attention(
        &[2., -2., 3., -3.],
        4,
        &cache,
        &mut [0.; 6],
        &mut out,
        &mut nb(),
    )
    .unwrap();
    let w = |score: f32| 1.0 / (1.0 + (-score).exp());
    let expected = [
        10. * w(4.) + 20. * (1. - w(4.)),
        10. * w(-4.) + 20. * (1. - w(-4.)),
        30. * w(-6.) + 40. * (1. - w(-6.)),
        30. * w(6.) + 40. * (1. - w(6.)),
    ];
    assert!(close(&out, &expected), "{out:?} vs {expected:?}");
}

#[test]
fn attention_refuses_empty_cache_bad_inputs_and_excess_work() {
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    let mut out = [7.; 2];
    assert_eq!(
        attention(&[1., 1.], 1, &cache, &mut [0.; 2], &mut out, &mut nb()),
        Err(Error::Shape)
    );
    cache.append(&[1., 1.], &[1., 1.]).unwrap();
    assert_eq!(
        attention(&[1., 1.], 1, &cache, &mut [0.; 2], &mut out, &mut nb()),
        Err(Error::Shape)
    );
    assert_eq!(
        attention(
            &[f32::NAN, 1.],
            1,
            &cache,
            &mut [0.; 3],
            &mut out,
            &mut nb()
        ),
        Err(Error::NonFinite)
    );
    assert_eq!(
        attention(
            &[f32::MAX, f32::MAX],
            1,
            &cache,
            &mut [0.; 3],
            &mut out,
            &mut nb()
        ),
        Err(Error::NonFinite)
    );
    assert_eq!(out, [7.; 2]);

    // One head of width 2 over 2 tokens costs 2 × 2 × 2 = 8 MACs.
    let (mut k, mut v) = ([0.; 4], [0.; 4]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    cache.append(&[1., 0.], &[1., 0.]).unwrap();
    cache.append(&[0., 1.], &[0., 1.]).unwrap();
    let mut out = [7.; 2];
    let mut budget = Budget::new(7, &NoCancel);
    assert_eq!(
        attention(&[1., 1.], 1, &cache, &mut [0.; 4], &mut out, &mut budget),
        Err(Error::WorkLimit)
    );
    assert_eq!((out, budget.remaining()), ([7.; 2], 7));
    let mut budget = Budget::new(8, &NoCancel);
    attention(&[1., 1.], 1, &cache, &mut [0.; 4], &mut out, &mut budget).unwrap();
    assert_eq!(budget.remaining(), 0);
}

#[test]
fn attention_observes_cancellation_before_work() {
    struct Cancelled;
    impl Cancel for Cancelled {
        fn cancelled(&self) -> bool {
            true
        }
    }
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    cache.append(&[1., 0.], &[1., 0.]).unwrap();
    let mut out = [7.; 2];
    assert_eq!(
        attention(
            &[1., 1.],
            1,
            &cache,
            &mut [0.; 3],
            &mut out,
            &mut Budget::new(100, &Cancelled)
        ),
        Err(Error::Cancelled)
    );
    assert_eq!(out, [7.; 2]);
}

#[test]
fn attention_observes_cancellation_between_heads() {
    // Polls: the up-front reservation, head 0, head 1. Cancel only at head 1,
    // after head 0 has been computed.
    struct Later(core::cell::Cell<u32>);
    impl Cancel for Later {
        fn cancelled(&self) -> bool {
            self.0.set(self.0.get() + 1);
            self.0.get() > 2
        }
    }
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    cache.append(&[1., 0.], &[1., 0.]).unwrap();
    let mut out = [7.; 4];
    let later = Later(core::cell::Cell::new(0));
    assert_eq!(
        attention(
            &[1., 1., 1., 1.],
            2,
            &cache,
            &mut [0.; 5],
            &mut out,
            &mut Budget::new(100, &later)
        ),
        Err(Error::Cancelled)
    );
    assert_eq!(later.0.get(), 3);
    assert_eq!(out, [7.; 4]);
}
