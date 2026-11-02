//! Compile the actual fixed-group authority/retirement mechanism without
//! privileged scheduler code. Real Engine identities and resources only.
#[path = "../../../kernel/src/m2/m3/window_retirement.rs"]
mod retirement;
use capability_core::{
    BlockScope, Engine, Error, Message, BLOCK_READ, PROCESS_FAULTED, RECV, SEND,
};
use retirement::{Authority, Phase};
const IMAGE: u32 = 0x5749_4e31;

fn channel(e: &mut Engine, owner: u64, sender: u64) -> (u64, u64) {
    let root = e.create_endpoint(owner).unwrap();
    let rx = e.derive(owner, root, RECV).unwrap();
    let tx = e.bootstrap_delegate(owner, root, sender, SEND).unwrap();
    e.close(owner, root).unwrap();
    (rx, tx)
}
fn run(test: impl FnOnce(&mut Engine, Authority, [u64; 4]) + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let mut e = Engine::new();
            for pid in 1..=8 {
                e.register(pid).unwrap();
            }
            let ingress = e.create_endpoint(4).unwrap();
            let apps = [2, 3].map(|pid| {
                let handles = core::array::from_fn(|index| {
                    if index % 2 == 0 {
                        e.bootstrap_delegate(4, ingress, pid, SEND).unwrap()
                    } else {
                        channel(&mut e, pid, 4).0
                    }
                });
                (pid, handles)
            });
            let authority = Authority {
                init: 1,
                launch: e.create_launch(1, IMAGE).unwrap(),
                image: IMAGE,
                process: e.create_process(1, 4).unwrap(),
                compositor: 4,
                apps,
            };
            // The actual four-endpoint Material owner quota: two private
            // presentation replies, document reply and supervisor control.
            let control = channel(&mut e, 2, 1).0;
            let document = channel(&mut e, 2, 7).0;
            let storage = e
                .create_block_device(
                    7,
                    BlockScope {
                        device: 0,
                        first_sector: 4,
                        sector_count: 580,
                    },
                )
                .unwrap();
            let entropy = e.create_entropy(8).unwrap();
            test(&mut e, authority, [control, document, storage, entropy]);
            assert!(e.check_invariants());
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn retirement_requires_exact_init_launch_dead_process_and_canonical_phase_without_mutation() {
    run(|e, authority, _| {
        let args = [authority.launch, authority.process, 1, 0, 0, 0];
        let before = e.clone();
        assert_eq!(retirement::authorize(e, 1, &authority, args), Err(-9));
        assert_eq!(*e, before);
        e.process_died(4, PROCESS_FAULTED, 6);
        e.exit(4).unwrap();
        let before = e.clone();
        for caller in [0, 2, 3, 5, 6, 7, 8] {
            assert!(retirement::authorize(e, caller, &authority, args).is_err());
        }
        for (index, value) in [
            (0, 0),
            (0, authority.process),
            (1, 0),
            (1, authority.launch),
            (2, 2),
            (3, 1),
            (4, 1),
            (5, 1),
        ] {
            let mut wrong = args;
            wrong[index] = value;
            assert!(retirement::authorize(e, 1, &authority, wrong).is_err());
        }
        let mut wrong = authority;
        wrong.image ^= 1;
        assert!(retirement::authorize(e, 1, &wrong, args).is_err());
        wrong = authority;
        wrong.compositor = 5;
        assert!(retirement::authorize(e, 1, &wrong, args).is_err());
        assert_eq!(
            retirement::authorize(e, 1, &authority, args),
            Ok(Phase::Retire)
        );
        assert_eq!(*e, before);
    });
}

#[test]
fn dead_peer_does_not_close_app_reply_but_explicit_retirement_does_without_other_authority_changes()
{
    run(|e, authority, preserved| {
        e.process_died(4, PROCESS_FAULTED, 6);
        e.exit(4).unwrap();
        for (pid, handles) in authority.apps {
            for index in [1, 3] {
                assert_eq!(e.receive(pid, handles[index]), Err(Error::Again));
            }
        }
        let before = e.stats();
        assert_eq!(
            retirement::authorize(
                e,
                1,
                &authority,
                [authority.launch, authority.process, 1, 0, 0, 0]
            ),
            Ok(Phase::Retire)
        );
        retirement::retire(e, &authority).unwrap();
        for (pid, handles) in authority.apps {
            for handle in handles {
                assert_eq!(e.send(pid, handle, &Message::new()), Err(Error::BadHandle));
                assert_eq!(e.receive(pid, handle), Err(Error::BadHandle));
            }
        }
        assert_eq!(e.stats().processes, before.processes);
        assert_eq!(e.stats().launch_objects, before.launch_objects);
        assert_eq!(e.stats().process_objects, before.process_objects);
        assert_eq!(e.stats().block_objects, before.block_objects);
        assert_eq!(e.stats().entropy_objects, before.entropy_objects);
        assert!(e.check_launch(1, authority.launch).is_ok());
        assert!(e
            .check_process(1, authority.process, capability_core::OBSERVE)
            .is_ok());
        assert!(e.check_endpoint(2, preserved[0], RECV).is_ok());
        assert!(e.check_endpoint(2, preserved[1], RECV).is_ok());
        assert!(e.check_block(7, preserved[2], BLOCK_READ).is_ok());
        assert!(e.check_entropy(8, preserved[3]).is_ok());
        // Repeated retirement is idempotent. Reused local slots are unrelated
        // generations and must remain live when the old manifest is retried.
        let replacement = e.create_endpoint(3).unwrap();
        retirement::retire(e, &authority).unwrap();
        assert!(e.check_endpoint(3, replacement, RECV).is_ok());
    });
}
