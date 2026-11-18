use super::*;

type Cache = Index<2, 128, 16>;

fn doc(object: u64, revision: u64, name: &str) -> Document<'_> {
    Document {
        object,
        revision,
        name: name.as_bytes(),
        collections: b"",
        metadata: b"",
        content: b"",
    }
}

#[test]
fn all_fields_utf8_and_updates_are_searchable() {
    let mut index = Cache::new();
    let grant = index.grant(0, 1).unwrap();
    index
        .update(
            grant,
            Document {
                collections: b"Work",
                metadata: b"type:note",
                content: "café\u{2003}東京".as_bytes(),
                ..doc(0, 1, "Meeting")
            },
        )
        .unwrap();
    let hits = index
        .query::<2>(&[grant], "MEETING work type:note café 東京".as_bytes())
        .unwrap();
    assert_eq!(
        hits.iter().copied().next(),
        Some(Hit {
            domain: 0,
            object: 0,
            revision: 1
        })
    );
    assert!(index.query::<2>(&[grant], b"meet").unwrap().is_empty());
    assert!(index
        .query::<2>(&[grant], "CAFÉ".as_bytes())
        .unwrap()
        .is_empty());
    index.update(grant, doc(0, 2, "Renamed")).unwrap();
    assert!(index.query::<2>(&[grant], b"Meeting").unwrap().is_empty());
    assert_eq!(index.query::<2>(&[grant], b"renamed").unwrap().len(), 1);
}

#[test]
fn fields_do_not_join_tokens() {
    let mut index = Cache::new();
    let grant = index.grant(0, 1).unwrap();
    index
        .update(
            grant,
            Document {
                collections: b"bar",
                ..doc(0, 1, "foo")
            },
        )
        .unwrap();
    assert_eq!(index.query::<2>(&[grant], b"foo bar").unwrap().len(), 1);
    assert!(index.query::<2>(&[grant], b"foobar").unwrap().is_empty());
}

#[test]
fn default_deny_revocation_and_epoch_replay() {
    let mut index = Cache::new();
    assert_eq!(index.grant(2, 1), Err(Error::Denied));
    assert_eq!(index.grant(0, 0), Err(Error::Denied));
    let old = index.grant(0, 7).unwrap();
    index.update(old, doc(0, 1, "secret")).unwrap();
    assert!(index.query::<2>(&[], b"secret").unwrap().is_empty());
    index.revoke(old).unwrap();
    assert_eq!(index.query::<2>(&[old], b"secret"), Err(Error::Denied));
    assert_eq!(index.grant(0, 7), Err(Error::Denied));
    assert_eq!(index.grant(0, 6), Err(Error::Denied));
    let new = index.grant(0, 8).unwrap();
    assert!(index.query::<2>(&[new], b"secret").unwrap().is_empty());
    assert_eq!(index.update(old, doc(0, 9, "secret")), Err(Error::Denied));
    assert_eq!(index.revoke(old), Err(Error::Denied));
    assert_eq!(index.rebuild(old, &[]), Err(Error::Denied));
}

#[test]
fn foreign_domain_and_stale_mutations_preserve_results() {
    let mut index = Cache::new();
    let grant = index.grant(0, 1).unwrap();
    index.update(grant, doc(0, 3, "retained")).unwrap();
    assert_eq!(
        index.update(grant, doc(2, 4, "foreign")),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(
        index.update(grant, doc(0, 3, "stale")),
        Err(Error::StaleRevision)
    );
    assert_eq!(index.delete(grant, 0, 2), Err(Error::StaleRevision));
    assert_eq!(
        index.update(grant, doc(1, 0, "zero")),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(index.query::<2>(&[grant], b"retained").unwrap().len(), 1);
    index.delete(grant, 0, 4).unwrap();
    assert!(index.query::<2>(&[grant], b"retained").unwrap().is_empty());
    assert_eq!(
        index.update(grant, doc(0, 4, "revived")),
        Err(Error::StaleRevision)
    );
    index.update(grant, doc(0, 5, "revived")).unwrap();
}

#[test]
fn domains_survive_other_domain_grants_rebuilds_and_revokes() {
    let mut index = Cache::new();
    let a = index.grant(0, 1).unwrap();
    index.update(a, doc(0, 1, "public")).unwrap();
    let b = index.grant(1, 1).unwrap();
    index.update(b, doc(2, 1, "private")).unwrap();
    assert_eq!(index.query::<2>(&[a], b"public").unwrap().len(), 1);
    assert!(index.query::<2>(&[a], b"private").unwrap().is_empty());
    index.rebuild(a, &[doc(0, 2, "replaced")]).unwrap();
    assert_eq!(index.query::<2>(&[b], b"private").unwrap().len(), 1);
    index.revoke(b).unwrap();
    assert_eq!(index.query::<2>(&[a, b], b"replaced"), Err(Error::Denied));
    assert_eq!(index.query::<2>(&[a], b"replaced").unwrap().len(), 1);
}

#[test]
fn invalid_utf8_and_bounded_text_tokens_queries_are_atomic() {
    let mut index = Index::<2, 8, 2>::new();
    let grant = index.grant(0, 1).unwrap();
    index.update(grant, doc(0, 1, "old")).unwrap();
    for (name, expected) in [
        (&b"123456789"[..], Error::TextFull),
        (&b"a b c"[..], Error::TokensFull),
        (&b"\xff"[..], Error::InvalidUtf8),
    ] {
        assert_eq!(
            index.update(
                grant,
                Document {
                    name,
                    ..doc(0, 2, "")
                }
            ),
            Err(expected)
        );
        assert_eq!(index.query::<1>(&[grant], b"old").unwrap().len(), 1);
        assert_eq!(index.query::<1>(&[grant], name), Err(expected));
    }
    assert_eq!(index.query::<1>(&[grant], b" \t"), Err(Error::EmptyQuery));
}

#[test]
fn table_tombstone_and_results_capacity_never_evicts() {
    let mut index = Index::<1, 16, 2>::new();
    let grant = index.grant(0, 1).unwrap();
    index.update(grant, doc(0, 1, "match")).unwrap();
    assert_eq!(
        index.rebuild(grant, &[doc(0, 2, "new"), doc(1, 1, "other")]),
        Err(Error::EntriesFull)
    );
    assert_eq!(
        index.update(grant, doc(1, 1, "other")),
        Err(Error::EntriesFull)
    );
    assert_eq!(
        index.query::<0>(&[grant], b"match"),
        Err(Error::ResultsFull)
    );
    assert_eq!(index.query::<1>(&[grant], b"match").unwrap().len(), 1);
    index.delete(grant, 0, 2).unwrap();
    assert_eq!(
        index.update(grant, doc(1, 1, "other")),
        Err(Error::EntriesFull)
    );
    let mut zero = Index::<0, 0, 0>::new();
    let grant = zero.grant(0, 1).unwrap();
    assert_eq!(zero.update(grant, doc(0, 1, "")), Err(Error::EntriesFull));
}

#[test]
fn complete_rebuild_is_atomic_and_retains_revision_floors() {
    let mut index = Cache::new();
    let grant = index.grant(0, 1).unwrap();
    index.update(grant, doc(0, 3, "old")).unwrap();
    assert_eq!(
        index.rebuild(grant, &[doc(0, 4, "new"), doc(2, 1, "foreign")]),
        Err(Error::InvalidIdentity)
    );
    assert_eq!(
        index.rebuild(grant, &[doc(0, 4, "new"), doc(0, 5, "duplicate")]),
        Err(Error::DuplicateObject)
    );
    assert_eq!(
        index.rebuild(grant, &[doc(0, 2, "stale")]),
        Err(Error::StaleRevision)
    );
    assert_eq!(index.query::<2>(&[grant], b"old").unwrap().len(), 1);
    index.rebuild(grant, &[doc(1, 1, "fresh")]).unwrap();
    assert!(index.query::<2>(&[grant], b"old").unwrap().is_empty());
    assert_eq!(
        index.update(grant, doc(0, 3, "stale")),
        Err(Error::StaleRevision)
    );
    assert_eq!(
        index.rebuild(grant, &[doc(0, 3, "stale")]),
        Err(Error::StaleRevision)
    );
    index.update(grant, doc(0, 4, "restored")).unwrap();
}

#[test]
fn corrupt_record_refuses_entire_query_and_recovers_from_catalogue() {
    let mut index = Cache::new();
    let grant = index.grant(0, 1).unwrap();
    index.update(grant, doc(0, 1, "match")).unwrap();
    index.update(grant, doc(1, 1, "match")).unwrap();
    assert_eq!(
        index.query::<1>(&[grant], b"match"),
        Err(Error::ResultsFull)
    );
    assert_eq!(
        index.query::<2>(&[grant, grant], b"match"),
        Err(Error::Denied)
    );
    assert_eq!(
        index.query::<2>(&[grant, grant, grant], b"match"),
        Err(Error::Denied)
    );
    index.entries[0][1].as_mut().unwrap().text.bytes[0] ^= 1;
    assert_eq!(index.query::<2>(&[grant], b"match"), Err(Error::Corrupt));
    assert_eq!(
        index.update(grant, doc(0, 2, "changed")),
        Err(Error::Corrupt)
    );
    index.rebuild(grant, &[doc(0, 2, "trusted")]).unwrap();
    assert_eq!(index.query::<2>(&[grant], b"trusted").unwrap().len(), 1);
    index.entries[0][0].as_mut().unwrap().text.count = usize::MAX;
    assert_eq!(index.query::<2>(&[grant], b"trusted"), Err(Error::Corrupt));
    index.rebuild(grant, &[]).unwrap();
    assert!(index.query::<2>(&[grant], b"trusted").unwrap().is_empty());
}

#[test]
fn denied_domain_corruption_cannot_leak_or_break_allowed_results() {
    let mut index = Cache::new();
    let a = index.grant(0, 1).unwrap();
    let b = index.grant(1, 1).unwrap();
    index.update(a, doc(0, 1, "allowed")).unwrap();
    index.update(b, doc(2, 1, "secret")).unwrap();
    let corrupt = index.entries[1][0].as_mut().unwrap();
    corrupt.hit.domain = 0;
    corrupt.text.count = usize::MAX;
    assert!(index.query::<2>(&[a], b"secret").unwrap().is_empty());
    assert_eq!(index.query::<2>(&[a], b"allowed").unwrap().len(), 1);
    assert_eq!(index.query::<2>(&[a, b], b"allowed"), Err(Error::Corrupt));
}

#[test]
fn input_bounds_precede_untrusted_text_scans_and_grant_iteration() {
    let mut index = Index::<2, 8, 2>::new();
    let grant = index.grant(0, 1).unwrap();
    assert_eq!(
        index.query::<1>(&[grant; DOMAINS + 1], b"ok"),
        Err(Error::Denied)
    );
    assert!(matches!(
        Text::<8, 2>::new(&[&[0xff; 9]]),
        Err(Error::TextFull)
    ));
}
