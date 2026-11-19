use nieuw_os_nas_client::{Checkpoint, Client, State, CHECKPOINT_BYTES, PENDING};
use nieuw_os_nas_core::{
    Entropy, Error, Principal, Service, Storage, Workspace, MAX_CIPHERTEXT, OBJECTS, SNAPSHOT_BYTES,
};

const HEADER: usize = 16;
const SLOT: usize = 64 + 2 * MAX_CIPHERTEXT;

fn encoded(client: &Client) -> [u8; CHECKPOINT_BYTES] {
    let mut bytes = [0xa5; CHECKPOINT_BYTES];
    client.checkpoint().encode(&mut bytes);
    bytes
}

fn reject(bytes: &[u8]) {
    assert!(matches!(Checkpoint::decode(bytes), Err(Error::Corrupt)));
}

#[test]
fn canonical_roundtrip_retains_maximum_edits_and_slot_holes() {
    let mut client = Client::new();
    for object in 0..PENDING {
        client
            .enqueue([3; 32], object, 7, &[object as u8; MAX_CIPHERTEXT])
            .unwrap();
    }
    client.take(1).unwrap();
    let bytes = encoded(&client);
    assert_eq!(bytes.len(), 8464);
    let restored = Client::restore(Checkpoint::decode(&bytes).unwrap());
    assert!(restored.edit(1).is_err());
    for object in [0, 2, 3] {
        let edit = restored.edit(object).unwrap();
        assert_eq!(edit.state(), State::NeedsReconcile);
        assert_eq!(edit.namespace(), &[3; 32]);
        assert_eq!(edit.object(), object);
        assert_eq!(edit.base_revision(), 7);
        assert_eq!(edit.local_ciphertext(), &[object as u8; MAX_CIPHERTEXT]);
    }
    assert_eq!(encoded(&restored), bytes);
    assert_eq!(
        encoded(&Client::restore(
            Checkpoint::decode(&encoded(&Client::new())).unwrap()
        )),
        encoded(&Client::new())
    );
}

#[test]
fn strict_length_header_state_bounds_and_padding_validation() {
    let mut client = Client::new();
    client.enqueue([3; 32], 0, 0, b"opaque").unwrap();
    let valid = encoded(&client);
    for length in 0..CHECKPOINT_BYTES {
        reject(&valid[..length]);
    }
    let mut extended = valid.to_vec();
    extended.push(0);
    reject(&extended);
    for at in [
        0,
        8,
        10,
        15,
        HEADER + 4,
        HEADER + 60,
        HEADER + 70,
        HEADER + SLOT,
    ] {
        let mut invalid = valid;
        invalid[at] ^= 0xff;
        reject(&invalid);
    }
    for state in [0, 2, 3, 4, 5, 6, 255] {
        let mut invalid = valid;
        invalid[HEADER] = state;
        reject(&invalid);
    }
    for (offset, value) in [
        (2, OBJECTS as u16),
        (56, 0),
        (56, (MAX_CIPHERTEXT + 1) as u16),
        (58, (MAX_CIPHERTEXT + 1) as u16),
    ] {
        let mut invalid = valid;
        invalid[HEADER + offset..HEADER + offset + 2].copy_from_slice(&value.to_le_bytes());
        reject(&invalid);
    }
    let mut invalid = valid;
    invalid[HEADER + 8..HEADER + 40].fill(0);
    reject(&invalid);
    let mut invalid = valid;
    invalid[HEADER + 40..HEADER + 48].copy_from_slice(&u64::MAX.to_le_bytes());
    reject(&invalid);
    let mut invalid = valid;
    invalid.copy_within(HEADER..HEADER + SLOT, HEADER + SLOT);
    reject(&invalid);
}

#[test]
fn absent_and_present_remote_versions_have_canonical_representations() {
    let mut client = Client::new();
    client.enqueue([3; 32], 0, 0, b"opaque").unwrap();
    let valid = encoded(&client);
    for offset in [1, 48, 58, 64 + MAX_CIPHERTEXT] {
        let mut invalid = valid;
        invalid[HEADER + offset] = 2;
        reject(&invalid);
    }
    let mut empty_remote = valid;
    empty_remote[HEADER + 1] = 1;
    let client = Client::restore(Checkpoint::decode(&empty_remote).unwrap());
    assert_eq!(
        client.edit(0).unwrap().remote_ciphertext(),
        Some((0, &[][..]))
    );
    assert_eq!(encoded(&client), empty_remote);
    let mut invalid = empty_remote;
    invalid[HEADER + 48] = 1;
    reject(&invalid);
    let mut invalid = empty_remote;
    invalid[HEADER + 58] = 1;
    invalid[HEADER + 64 + MAX_CIPHERTEXT] = 1;
    reject(&invalid);
}

struct TestEntropy(u8);
impl Entropy for TestEntropy {
    fn fill(&mut self, bytes: &mut [u8; 32]) -> Result<(), Error> {
        self.0 += 1;
        bytes.fill(self.0);
        Ok(())
    }
}

#[derive(Default)]
struct Memory(Option<Vec<u8>>);
impl Storage for Memory {
    fn load(&mut self, out: &mut [u8; SNAPSHOT_BYTES]) -> Result<Option<usize>, Error> {
        if let Some(bytes) = &self.0 {
            out.copy_from_slice(bytes);
            Ok(Some(bytes.len()))
        } else {
            Ok(None)
        }
    }
    fn replace(&mut self, bytes: &[u8; SNAPSHOT_BYTES]) -> Result<(), Error> {
        self.0 = Some(bytes.to_vec());
        Ok(())
    }
}

#[test]
fn publication_and_live_tickets_never_survive_encoded_restore() {
    let admin = Principal([1; 32]);
    let owner = Principal([2; 32]);
    let mut work = Box::new(Workspace::new());
    let mut entropy = TestEntropy(10);
    let mut service =
        Service::initialize(Memory::default(), admin, &mut entropy, &mut work).unwrap();
    let invite = service
        .invite(admin, owner, 100, 100, 10, &mut entropy)
        .unwrap();
    service.accept(owner, &invite, 100).unwrap();
    let mut a = Client::new();
    let mut b = Client::new();
    let x = a.enqueue(invite.namespace, 0, 0, b"local A").unwrap();
    let y = b.enqueue(invite.namespace, 0, 0, b"local B").unwrap();
    a.prepare(x, &mut service, owner).unwrap();
    let mut restored = Client::restore(Checkpoint::decode(&encoded(&a)).unwrap());
    assert_eq!(restored.edit(x).unwrap().state(), State::NeedsReconcile);
    assert_eq!(
        restored
            .commit(x, &mut service, owner)
            .unwrap_err()
            .operation,
        Error::ReopenRequired
    );
    a.commit(x, &mut service, owner).unwrap();
    let mut restored = Client::restore(Checkpoint::decode(&encoded(&a)).unwrap());
    assert_eq!(restored.edit(x).unwrap().state(), State::NeedsReconcile);
    assert_eq!(
        restored.reconcile_latest(x, &mut service, owner),
        Ok(State::Observed(1))
    );
    let restored = Client::restore(Checkpoint::decode(&encoded(&restored)).unwrap());
    assert_eq!(restored.edit(x).unwrap().state(), State::NeedsReconcile);
    assert_eq!(
        restored.edit(x).unwrap().remote_ciphertext(),
        Some((1, b"local A".as_slice()))
    );
    assert_eq!(
        b.reconcile_latest(y, &mut service, owner),
        Ok(State::Conflict)
    );
    let restored = Client::restore(Checkpoint::decode(&encoded(&b)).unwrap());
    assert_eq!(restored.edit(y).unwrap().state(), State::NeedsReconcile);
    assert_eq!(restored.edit(y).unwrap().local_ciphertext(), b"local B");
    assert_eq!(
        restored.edit(y).unwrap().remote_ciphertext(),
        Some((1, b"local A".as_slice()))
    );
}
