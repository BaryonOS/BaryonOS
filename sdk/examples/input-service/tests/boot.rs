//! Bootstrap/registry bytes only; numeric test handles prove no authority.
use nieuw_os_native_input_service::{
    boot::{self, PolicyBoot, SourceBoot},
    policy::Policy,
};

fn raw(slot: u64) -> u64 {
    (1 << 32) | slot
}
fn source() -> [u64; 128] {
    let mut w = [0; 128];
    for (offset, value) in [
        (32, boot::SOURCE_MAGIC),
        (40, 1),
        (48, raw(1)),
        (56, raw(2)),
        (64, raw(3)),
        (72, 10),
        (80, 20),
        (88, raw(4)),
        (96, 30),
    ] {
        w[offset / 8] = value;
    }
    w
}
fn policy() -> [u64; 128] {
    let mut w = [0; 128];
    for (offset, value) in [
        (32, boot::POLICY_MAGIC),
        (40, 2),
        (48, raw(1)),
        (56, raw(2)),
        (64, raw(3)),
        (72, raw(4)),
        (80, 1),
        (88, 2),
        (96, 3),
        (104, 20),
        (112, 800),
        (120, 600),
        (128, raw(5)),
        (136, raw(6)),
        (144, raw(7)),
        (152, raw(8)),
        (160, 1),
        (168, 101),
        (176, 4),
        (184, raw(9)),
    ] {
        w[offset / 8] = value;
    }
    w
}
#[test]
fn source_bootstrap_rejects_unknown_generation_reserved_state_and_handle_shape() {
    let good = source();
    let parsed = SourceBoot::decode(&good).unwrap();
    assert_eq!(parsed.configuration.policy_pid, 10);
    assert_eq!(parsed.generation, 1);
    for (offset, value) in [
        (32, 0),
        (40, 0),
        (48, 0),
        (56, u64::MAX),
        (72, 0),
        (80, 0),
        (96, 10),
        (104, 1),
        (1008, 1),
    ] {
        let mut bad = good;
        bad[offset / 8] = value;
        assert!(SourceBoot::decode(&bad).is_err(), "offset{offset}");
    }
}
#[test]
fn policy_bootstrap_geometry_bindings_four_endpoints_and_unused_triplets_are_checked() {
    let good = policy();
    let parsed = PolicyBoot::decode(&good).unwrap();
    assert!(Policy::new(
        parsed.configuration,
        parsed.replies,
        &parsed.clients[..parsed.client_count]
    )
    .is_ok());
    for (offset, value) in [
        (32, 0),
        (40, 0),
        (48, 0),
        (112, 1 << 32),
        (120, 1 << 32),
        (160, 0),
        (160, 9),
        (192, 1),
        (1016, 1),
    ] {
        let mut bad = good;
        bad[offset / 8] = value;
        assert!(PolicyBoot::decode(&bad).is_err(), "offset{offset}");
    }
    for (offset, value) in [(56, raw(1)), (88, 1), (104, 0), (112, 0), (176, 1)] {
        let mut bad = good;
        bad[offset / 8] = value;
        let parsed = PolicyBoot::decode(&bad).unwrap();
        assert!(
            Policy::new(
                parsed.configuration,
                parsed.replies,
                &parsed.clients[..parsed.client_count]
            )
            .is_err(),
            "offset{offset}"
        );
    }
}
#[test]
fn ready_is_exactly_bound_to_role_service_boot_and_real_raw_epoch() {
    for role in [1, 2] {
        let raw_epoch = u64::from(role == 1) * 71;
        let out = boot::ready(role, 20, 5, raw_epoch).unwrap();
        let envelope = out.as_bytes();
        assert_eq!(u32::from_le_bytes(envelope[16..20].try_into().unwrap()), 32);
        assert_eq!(
            u64::from_le_bytes(envelope[8..16].try_into().unwrap()),
            boot::READY_TAG
        );
        let bytes = &envelope[64..96];
        assert_eq!(&bytes[..4], b"IPD1");
        assert_eq!(&bytes[4..6], &1u16.to_le_bytes());
        assert_eq!(&bytes[6..8], &role.to_le_bytes());
        assert_eq!(&bytes[8..16], &20u64.to_le_bytes());
        assert_eq!(&bytes[16..24], &5u64.to_le_bytes());
        assert_eq!(&bytes[24..32], &raw_epoch.to_le_bytes());
    }
    for (role, epoch, generation, raw_epoch) in [
        (0, 20, 5, 1),
        (3, 20, 5, 1),
        (1, 0, 5, 1),
        (1, 20, 0, 1),
        (1, 20, 5, 0),
        (2, 20, 5, 1),
    ] {
        assert!(boot::ready(role, epoch, generation, raw_epoch).is_err());
    }
}
