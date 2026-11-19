use super::*;

const ACTOR: Actor = Actor {
    app: 1,
    pid: 10,
    epoch: 2,
};
const OBJECT: Object = Object { id: 3, revision: 4 };
const READ: Rights = Rights {
    read: true,
    write: false,
    export: false,
};
const ALL: Rights = Rights {
    read: true,
    write: true,
    export: true,
};

fn table() -> Grants<2, 4> {
    let mut table = Grants::new(1);
    table.register_object(OBJECT).unwrap();
    table
}

#[test]
fn only_explicit_rights_authorize_the_exact_object() {
    for rights in [
        READ,
        Rights {
            write: true,
            ..READ
        },
        Rights {
            read: false,
            write: true,
            export: false,
        },
        Rights {
            read: false,
            write: false,
            export: true,
        },
        ALL,
    ] {
        let mut table = table();
        let handle = table
            .grant(ACTOR, OBJECT, rights, Duration::Session)
            .unwrap();
        for (operation, permitted) in [
            (Operation::Read, rights.read),
            (Operation::Write, rights.write),
            (Operation::Export, rights.export),
        ] {
            assert_eq!(
                table.authorize(handle, ACTOR, OBJECT, operation),
                if permitted {
                    Ok(())
                } else {
                    Err(Error::Denied)
                }
            );
        }
        let other = Object { id: 5, revision: 4 };
        table.register_object(other).unwrap();
        assert_eq!(
            table.authorize(handle, ACTOR, other, Operation::Read),
            Err(Error::WrongObject)
        );
    }
}

#[test]
fn new_apps_internet_only_apps_helpers_and_stale_epochs_are_denied() {
    let mut table = table();
    let handle = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    // Network permissions and explanation preferences are deliberately not inputs.
    for actor in [
        Actor { app: 2, ..ACTOR },
        Actor { app: 3, ..ACTOR },
        Actor { pid: 11, ..ACTOR },
        Actor { epoch: 1, ..ACTOR },
        Actor { epoch: 3, ..ACTOR },
    ] {
        assert_eq!(
            table.authorize(handle, actor, OBJECT, Operation::Read),
            Err(Error::WrongActor)
        );
    }
}

#[test]
fn unknown_objects_and_revisions_never_register_implicitly() {
    let mut table = table();
    let handle = table.grant(ACTOR, OBJECT, READ, Duration::Session).unwrap();
    for (object, error) in [
        (Object { id: 99, ..OBJECT }, Error::UnknownObject),
        (
            Object {
                revision: 3,
                ..OBJECT
            },
            Error::StaleRevision,
        ),
        (
            Object {
                revision: 5,
                ..OBJECT
            },
            Error::StaleRevision,
        ),
    ] {
        assert_eq!(
            table.grant(ACTOR, object, ALL, Duration::Session),
            Err(error)
        );
        assert_eq!(
            table.authorize(handle, ACTOR, object, Operation::Read),
            Err(error)
        );
        assert_eq!(table.close(ACTOR, object), Err(error));
    }
    assert_eq!(
        table.advance_revision(Object { id: 99, ..OBJECT }),
        Err(Error::UnknownObject)
    );
    assert_eq!(
        table.authorize(handle, ACTOR, OBJECT, Operation::Read),
        Ok(())
    );
}

#[test]
fn revoke_and_slot_reuse_never_resurrect_retained_handles() {
    let mut table = table();
    let old = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    table.revoke(old).unwrap();
    let new = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    assert_eq!(old.slot, new.slot);
    assert_ne!(old.generation, new.generation);
    assert_eq!(table.revoke(old), Err(Error::StaleHandle));
    assert_eq!(
        table.authorize(old, ACTOR, OBJECT, Operation::Read),
        Err(Error::StaleHandle)
    );
    assert_eq!(table.authorize(new, ACTOR, OBJECT, Operation::Read), Ok(()));
}

#[test]
fn close_retires_all_matching_until_close_grants_only() {
    let mut table = table();
    let other = Object { id: 5, ..OBJECT };
    table.register_object(other).unwrap();
    let a = table
        .grant(ACTOR, OBJECT, READ, Duration::UntilClose)
        .unwrap();
    let b = table
        .grant(ACTOR, OBJECT, ALL, Duration::UntilClose)
        .unwrap();
    let session = table.grant(ACTOR, OBJECT, READ, Duration::Session).unwrap();
    let unrelated = table
        .grant(ACTOR, other, READ, Duration::UntilClose)
        .unwrap();
    table.close(Actor { pid: 11, ..ACTOR }, OBJECT).unwrap();
    assert_eq!(table.authorize(a, ACTOR, OBJECT, Operation::Read), Ok(()));
    table.close(ACTOR, OBJECT).unwrap();
    for handle in [a, b] {
        assert_eq!(
            table.authorize(handle, ACTOR, OBJECT, Operation::Read),
            Err(Error::StaleHandle)
        );
    }
    assert_eq!(
        table.authorize(session, ACTOR, OBJECT, Operation::Read),
        Ok(())
    );
    assert_eq!(
        table.authorize(unrelated, ACTOR, other, Operation::Read),
        Ok(())
    );
    let reopened = table
        .grant(ACTOR, OBJECT, READ, Duration::UntilClose)
        .unwrap();
    assert_ne!(a, reopened);
    assert_eq!(table.revoke(a), Err(Error::StaleHandle));
}

#[test]
fn session_end_is_permanent_and_invalidates_both_durations() {
    let mut table = table();
    let a = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    let b = table
        .grant(ACTOR, OBJECT, ALL, Duration::UntilClose)
        .unwrap();
    table.end_session();
    table.end_session();
    for handle in [a, b] {
        assert_eq!(
            table.authorize(handle, ACTOR, OBJECT, Operation::Read),
            Err(Error::SessionEnded)
        );
        assert_eq!(table.revoke(handle), Err(Error::SessionEnded));
    }
    assert_eq!(
        table.grant(ACTOR, OBJECT, ALL, Duration::Session),
        Err(Error::SessionEnded)
    );
    assert_eq!(table.register_object(OBJECT), Err(Error::SessionEnded));
    assert_eq!(
        table.advance_revision(Object {
            revision: 5,
            ..OBJECT
        }),
        Err(Error::SessionEnded)
    );
    assert_eq!(table.close(ACTOR, OBJECT), Err(Error::SessionEnded));
    assert_eq!(table.retire_process(ACTOR), Err(Error::SessionEnded));
}

#[test]
fn handles_cannot_cross_session_tables_or_use_invalid_slots() {
    let mut a = table();
    let mut b = Grants::<2, 4>::new(2);
    b.register_object(OBJECT).unwrap();
    let handle = a.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    let valid = b.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    assert_eq!(b.revoke(handle), Err(Error::StaleHandle));
    assert_eq!(
        b.authorize(handle, ACTOR, OBJECT, Operation::Read),
        Err(Error::StaleHandle)
    );
    for forged in [
        Handle {
            slot: usize::MAX,
            ..valid
        },
        Handle {
            generation: 0,
            ..valid
        },
    ] {
        assert_eq!(b.revoke(forged), Err(Error::StaleHandle));
        assert_eq!(
            b.authorize(forged, ACTOR, OBJECT, Operation::Read),
            Err(Error::StaleHandle)
        );
    }
    assert_eq!(b.authorize(valid, ACTOR, OBJECT, Operation::Read), Ok(()));
}

#[test]
fn capacity_failures_preserve_live_grants_and_objects() {
    let mut table = Grants::<1, 1>::new(1);
    table.register_object(OBJECT).unwrap();
    let handle = table.grant(ACTOR, OBJECT, READ, Duration::Session).unwrap();
    assert_eq!(
        table.register_object(Object { id: 5, ..OBJECT }),
        Err(Error::ObjectTableFull)
    );
    assert_eq!(
        table.register_object(Object {
            revision: 5,
            ..OBJECT
        }),
        Err(Error::AlreadyRegistered)
    );
    assert_eq!(
        table.grant(ACTOR, OBJECT, ALL, Duration::Session),
        Err(Error::GrantTableFull)
    );
    assert_eq!(table.generation, handle.generation);
    assert_eq!(
        table.authorize(handle, ACTOR, OBJECT, Operation::Read),
        Ok(())
    );
    assert_eq!(
        table.authorize(handle, ACTOR, OBJECT, Operation::Export),
        Err(Error::Denied)
    );
    let mut zero = Grants::<0, 0>::new(2);
    assert_eq!(zero.register_object(OBJECT), Err(Error::ObjectTableFull));
    assert_eq!(
        zero.grant(ACTOR, OBJECT, READ, Duration::Session),
        Err(Error::UnknownObject)
    );
    let mut no_grants = Grants::<1, 0>::new(3);
    no_grants.register_object(OBJECT).unwrap();
    assert_eq!(
        no_grants.grant(ACTOR, OBJECT, READ, Duration::Session),
        Err(Error::GrantTableFull)
    );
}

#[test]
fn generation_overflow_denies_issuance_without_reuse_or_loss_of_revocation() {
    let mut table = table();
    let first = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    table.generation = u64::MAX - 1;
    let last = table.grant(ACTOR, OBJECT, READ, Duration::Session).unwrap();
    assert_eq!(last.generation, u64::MAX);
    for _ in 0..2 {
        assert_eq!(
            table.grant(ACTOR, OBJECT, ALL, Duration::Session),
            Err(Error::GenerationExhausted)
        );
    }
    assert_eq!(
        table.authorize(first, ACTOR, OBJECT, Operation::Export),
        Ok(())
    );
    table.revoke(first).unwrap();
    table.revoke(last).unwrap();
    assert_eq!(
        table.grant(ACTOR, OBJECT, ALL, Duration::Session),
        Err(Error::GenerationExhausted)
    );
    assert_eq!(
        table.authorize(last, ACTOR, OBJECT, Operation::Read),
        Err(Error::StaleHandle)
    );
}

#[test]
fn revisions_advance_without_rollback_or_old_handle_revival() {
    let mut table = table();
    let old = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    assert_eq!(table.advance_revision(OBJECT), Err(Error::StaleRevision));
    assert_eq!(table.authorize(old, ACTOR, OBJECT, Operation::Read), Ok(()));
    let next = Object {
        revision: u64::MAX,
        ..OBJECT
    };
    table.advance_revision(next).unwrap();
    assert_eq!(
        table.authorize(old, ACTOR, next, Operation::Read),
        Err(Error::StaleHandle)
    );
    assert_eq!(
        table.grant(ACTOR, OBJECT, ALL, Duration::Session),
        Err(Error::StaleRevision)
    );
    let new = table.grant(ACTOR, next, ALL, Duration::Session).unwrap();
    assert_eq!(table.advance_revision(OBJECT), Err(Error::StaleRevision));
    assert_eq!(table.advance_revision(next), Err(Error::StaleRevision));
    assert_eq!(table.authorize(new, ACTOR, next, Operation::Write), Ok(()));
}

#[test]
fn process_retirement_invalidates_all_its_grants_without_affecting_other_apps() {
    let mut table = table();
    let old = table.grant(ACTOR, OBJECT, ALL, Duration::Session).unwrap();
    let close = table
        .grant(ACTOR, OBJECT, ALL, Duration::UntilClose)
        .unwrap();
    let other = Actor { app: 2, ..ACTOR };
    let retained = table.grant(other, OBJECT, READ, Duration::Session).unwrap();
    table.retire_process(ACTOR).unwrap();
    for handle in [old, close] {
        assert_eq!(
            table.authorize(handle, ACTOR, OBJECT, Operation::Read),
            Err(Error::StaleHandle)
        );
    }
    assert_eq!(
        table.authorize(retained, other, OBJECT, Operation::Read),
        Ok(())
    );
    let restarted = Actor { epoch: 3, ..ACTOR };
    let fresh = table
        .grant(restarted, OBJECT, READ, Duration::Session)
        .unwrap();
    assert_eq!(
        table.authorize(fresh, ACTOR, OBJECT, Operation::Read),
        Err(Error::WrongActor)
    );
    assert_eq!(
        table.authorize(fresh, restarted, OBJECT, Operation::Read),
        Ok(())
    );
}

#[test]
fn empty_rights_do_not_consume_capacity_or_generations() {
    let mut table = table();
    let none = Rights {
        read: false,
        write: false,
        export: false,
    };
    assert_eq!(
        table.grant(ACTOR, OBJECT, none, Duration::Session),
        Err(Error::EmptyRights)
    );
    assert_eq!(table.generation, 0);
    assert!(table.grants.iter().all(Option::is_none));
}
