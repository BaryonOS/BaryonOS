//! Per-window input forwarding. The caller supplies authenticated kernel PIDs
//! and events decoded by the existing input service, never app-authored events.
use nieuw_os_input_core::{
    wire::{self, Operation, Request},
    CancelReason, Error, Event,
};

const CAPACITY: usize = 16;
#[derive(Clone, Copy)]
struct Item {
    sequence: u64,
    generation: u64,
    event: Event,
}
pub struct Queue {
    retired: bool,
    pid: u64,
    id: u64,
    epoch: u64,
    last_request: u64,
    next: u64,
    generation: u64,
    offered: Option<u64>,
    items: [Option<Item>; CAPACITY],
    head: usize,
    length: usize,
}
impl Queue {
    /// An offered event or focus cancellation must reach its real app ACK.
    pub fn has_pending(&self) -> bool {
        self.length != 0 || self.offered.is_some()
    }
    pub fn new(pid: u64, id: u64, epoch: u64) -> Result<Self, Error> {
        if pid == 0 || id == 0 || epoch == 0 {
            return Err(Error::Configuration);
        }
        Ok(Self {
            retired: false,
            pid,
            id,
            epoch,
            last_request: 0,
            next: 1,
            generation: 1,
            offered: None,
            items: [None; CAPACITY],
            head: 0,
            length: 0,
        })
    }
    pub fn cancel(&mut self, reason: CancelReason) -> Result<(), Error> {
        if self.retired {
            return Ok(());
        }
        let next_generation = self
            .generation
            .checked_add(1)
            .ok_or(Error::CounterExhausted)?;
        self.next.checked_add(1).ok_or(Error::CounterExhausted)?;
        self.items = [None; CAPACITY];
        self.head = 0;
        self.length = 0;
        self.offered = None;
        self.push(Event::Cancel(reason))?;
        self.generation = next_generation;
        Ok(())
    }
    pub fn push(&mut self, event: Event) -> Result<(), Error> {
        if self.retired {
            return Ok(());
        }
        if self.length == CAPACITY {
            return self.cancel(CancelReason::QueueFull);
        }
        let sequence = self.next;
        self.next = self.next.checked_add(1).ok_or(Error::CounterExhausted)?;
        self.items[(self.head + self.length) % CAPACITY] = Some(Item {
            sequence,
            generation: self.generation,
            event,
        });
        self.length += 1;
        Ok(())
    }
    /// Wrong senders cannot advance correlation state or receive a reply.
    pub fn handle(&mut self, sender: u64, bytes: &[u8]) -> Result<[u8; wire::SIZE], Error> {
        if sender != self.pid {
            return Err(Error::Access);
        }
        if self.retired {
            return Err(Error::ActorDead);
        }
        let request = Request::decode(bytes, self.epoch)?;
        if !matches!(request.operation, Operation::Peek | Operation::Ack) {
            return Ok(wire::failure(request, self.id, Error::WrongRole));
        }
        if request.id <= self.last_request {
            return Ok(wire::failure(request, self.id, Error::StaleRequest));
        }
        self.last_request = request.id;
        let event = match request.operation {
            Operation::Peek => {
                let item = self.items[self.head];
                self.offered = item.map(|i| i.sequence);
                item.map(|i| (i.sequence, i.generation, i.event))
            }
            Operation::Ack => {
                if self.offered != Some(request.arg0)
                    || self.items[self.head].is_none_or(|i| i.sequence != request.arg0)
                {
                    return Ok(wire::failure(request, self.id, Error::StaleAck));
                }
                self.items[self.head] = None;
                self.head = (self.head + 1) % CAPACITY;
                self.length -= 1;
                self.offered = None;
                None
            }
            _ => return Err(Error::WrongRole),
        };
        Ok(wire::response(request, self.id, event))
    }
    pub fn retire(&mut self) {
        self.retired = true;
        self.items = [None; CAPACITY];
        self.head = 0;
        self.length = 0;
        self.offered = None;
    }
    /// Rebuild a prepared PEEK immediately before a delayed send. Focus loss
    /// or overflow may have superseded an offer while the reply queue was full.
    /// This does not consume another request ID or acknowledge any event.
    pub fn refresh(&mut self, prepared: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Error> {
        let operation = u16::from_le_bytes([prepared[6], prepared[7]]) & !wire::REPLY;
        let status = u64::from_le_bytes(prepared[24..32].try_into().unwrap());
        if operation != Operation::Peek as u16 || status > 1 {
            return Ok(*prepared);
        }
        let mut bytes = *prepared;
        bytes[6..8].copy_from_slice(&operation.to_le_bytes());
        bytes[24..].fill(0);
        let request = Request::decode(&bytes, self.epoch)?;
        if request.id != self.last_request {
            return Err(Error::StaleRequest);
        }
        if request.operation != Operation::Peek {
            return Ok(*prepared);
        }
        let item = self.items[self.head];
        self.offered = item.map(|i| i.sequence);
        Ok(wire::response(
            request,
            self.id,
            item.map(|i| (i.sequence, i.generation, i.event)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req(operation: Operation, id: u64, arg0: u64) -> [u8; wire::SIZE] {
        Request {
            operation,
            id,
            epoch: 7,
            arg0,
            arg1: 0,
        }
        .encode()
    }
    fn word(b: &[u8], at: usize) -> u64 {
        u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
    }
    #[test]
    fn actual_sender_scope_and_epoch_precede_state_changes() {
        let mut q = Queue::new(42, 1, 7).unwrap();
        let p = req(Operation::Peek, 1, 0);
        assert_eq!(q.handle(43, &p), Err(Error::Access));
        let mut bad = p;
        bad[16] = 8;
        assert_eq!(q.handle(42, &bad), Err(Error::WrongEpoch));
        assert_eq!(word(&q.handle(42, &p).unwrap(), 24), 1);
        assert_eq!(
            word(&q.handle(42, &p).unwrap(), 24),
            Error::StaleRequest as u64
        );
    }
    #[test]
    fn cancellation_supersedes_old_offer_without_consuming_new_event() {
        let mut q = Queue::new(42, 1, 7).unwrap();
        q.push(Event::Cancel(CancelReason::Resync)).unwrap();
        let offer = q.handle(42, &req(Operation::Peek, 1, 0)).unwrap();
        let old = word(&offer, 48);
        q.cancel(CancelReason::FocusLost).unwrap();
        assert_eq!(
            word(&q.handle(42, &req(Operation::Ack, 2, old)).unwrap(), 24),
            Error::StaleAck as u64
        );
        let new = q.handle(42, &req(Operation::Peek, 3, 0)).unwrap();
        assert!(word(&new, 48) > old);
        assert_eq!(new[41], 1);
        assert_eq!(
            word(
                &q.handle(42, &req(Operation::Ack, 4, word(&new, 48)))
                    .unwrap(),
                24
            ),
            0
        );
        assert_eq!(
            word(&q.handle(42, &req(Operation::Peek, 5, 0)).unwrap(), 24),
            1
        );
    }
    #[test]
    fn overflow_cancels_instead_of_delivering_partial_input() {
        let mut q = Queue::new(42, 1, 7).unwrap();
        for _ in 0..=CAPACITY {
            q.push(Event::Cancel(CancelReason::Resync)).unwrap();
        }
        let p = q.handle(42, &req(Operation::Peek, 1, 0)).unwrap();
        assert_eq!(p[41], 3);
        assert_eq!(q.length, 1);
        assert_eq!(
            word(&q.handle(42, &req(Operation::Focus, 2, 0)).unwrap(), 24),
            Error::WrongRole as u64
        );
    }
    #[test]
    fn delayed_reply_refresh_never_delivers_superseded_offer() {
        let mut q = Queue::new(42, 1, 7).unwrap();
        q.push(Event::Cancel(CancelReason::Resync)).unwrap();
        let old = q.handle(42, &req(Operation::Peek, 1, 0)).unwrap();
        q.cancel(CancelReason::FocusLost).unwrap();
        let fresh = q.refresh(&old).unwrap();
        assert!(word(&fresh, 48) > word(&old, 48));
        assert_eq!(fresh[41], 1);
        let ack = q
            .handle(42, &req(Operation::Ack, 2, word(&fresh, 48)))
            .unwrap();
        assert_eq!(q.refresh(&ack).unwrap(), ack);
        let empty = q.handle(42, &req(Operation::Peek, 3, 0)).unwrap();
        q.push(Event::Key(ui_core::KeyboardEvent {
            key: ui_core::PhysicalKey::A,
            action: ui_core::KeyAction::Press,
            modifiers: ui_core::KeyboardModifiers::default(),
        }))
        .unwrap();
        let key = q.refresh(&empty).unwrap();
        assert_eq!(key[40], 1);
        assert!(word(&key, 56) > word(&fresh, 56));
    }
}
