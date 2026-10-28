//! Actual scalar ring-3 adapter. Only this file invokes the native block/IPC ABI.
//! There is no allocator, automatic formatting, network access or write replay.
use crate::{journal, rt, wire};
use wire::{Status, DELETE, GET, PUT};

fn poll(cap: u64, ticket: u64, length: usize) -> Result<rt::Message, Status> {
    for _ in 0..rt::LIMIT {
        let (code, amount) = rt::call(0x41, [cap, ticket, rt::OUT, 320, 0, 0]);
        if code == -8 {
            rt::yield_now();
            continue;
        }
        if code != 0 {
            return Err(Status::from_code(code));
        }
        let message = rt::output();
        if amount != length as u64
            || message.sender() != 0
            || message.tag() != ticket
            || message.length() != length
            || message.cap_count() != 0
            || message.bytes[24..64].iter().any(|byte| *byte != 0)
            || message.bytes[64 + length..].iter().any(|byte| *byte != 0)
        {
            return Err(Status::Protocol);
        }
        return Ok(message);
    }
    Err(Status::Timeout)
}
fn submit(number: u64, args: [u64; 6]) -> Result<u64, Status> {
    let (code, ticket) = rt::call(number, args);
    if code == 0 {
        Ok(ticket)
    } else {
        if ticket != 0 {
            rt::fail(70);
        }
        Err(Status::from_code(code))
    }
}
fn flush(cap: u64) -> Result<(), Status> {
    let ticket = submit(0x43, [cap, 0, 0, 0, 0, 0])?;
    poll(cap, ticket, 0)?;
    Ok(())
}
fn read_sector(cap: u64, index: u64) -> Result<[u8; journal::SECTOR], Status> {
    let mut record = [0; journal::SECTOR];
    for part in 0..4 {
        let ticket = submit(0x40, [cap, index, (part * 128) as u64, 128, 0, 0])?;
        let message = poll(cap, ticket, 128)?;
        record[part * 128..part * 128 + 128].copy_from_slice(&message.bytes[64..192]);
    }
    Ok(record)
}
fn write_record(cap: u64, index: u64, record: &[u8; journal::SECTOR]) -> Result<(), Status> {
    for (index, byte) in record.iter().enumerate() {
        // SAFETY: The native manifest maps our private, writable SECTOR page.
        // Exactly512 bytes fit in it; one thread owns it and no Rust reference
        // to the page exists. Kernel WRITE copies it before any scheduling.
        unsafe {
            (rt::SECTOR as *mut u8).add(index).write_volatile(*byte);
        }
    }
    let ticket = submit(0x42, [cap, index, rt::SECTOR, journal::SECTOR as u64, 0, 0])?;
    poll(cap, ticket, 0)?;
    Ok(())
}
fn status(error: journal::Error) -> Status {
    match error {
        journal::Error::Invalid => Status::Invalid,
        journal::Error::Range => Status::Range,
        journal::Error::Stale => Status::Stale,
        journal::Error::Full => Status::Full,
        journal::Error::Corrupt => Status::Corrupt,
        journal::Error::NotFound => Status::NotFound,
    }
}
fn startup_error(error: Status) -> ! {
    rt::put(232, error as i32 as i64 as u64);
    rt::put(40, 1);
    rt::worker()
}

pub fn run() -> ! {
    let epoch = rt::word(32);
    let cap = rt::word(48);
    let roots = [rt::word(56), rt::word(80)];
    let replies = [rt::word(88), rt::word(72)];
    let clients = [rt::word(160), rt::word(168)];
    if epoch == 0
        || cap == 0
        || roots.contains(&0)
        || replies.contains(&0)
        || clients.contains(&0)
        || roots[0] == roots[1]
        || replies[0] == replies[1]
        || clients[0] == clients[1]
    {
        startup_error(Status::Invalid);
    }
    let mut recovery = journal::Recovery::new();
    for index in 0..journal::SLOTS {
        let record = match read_sector(cap, index as u64) {
            Ok(record) => record,
            Err(error) => startup_error(error),
        };
        if recovery.push(index, &record).is_err() {
            startup_error(Status::Corrupt);
        }
    }
    let mut current = match recovery.finish() {
        Ok(current) => current,
        Err(_) => startup_error(Status::Corrupt),
    };
    if let Err(error) = flush(cap) {
        startup_error(error);
    }
    rt::put(128, current.sequence());
    rt::put(232, 0);
    rt::put(192, 1);
    let mut uncertain = false;
    loop {
        for (domain, root) in roots.iter().copied().enumerate() {
            let Some(mut message) = rt::receive(root) else {
                continue;
            };
            // Capabilities are actual received handles, and must be closed even
            // when another structural/caller check rejects this request.
            let had_caps = message.close_caps();
            if message.sender() != clients[domain] {
                continue;
            }
            let mut body = [0; wire::SIZE];
            body.copy_from_slice(&message.bytes[64..64 + wire::SIZE]);
            let header = wire::header(&body);
            let validation =
                if had_caps || message.tag() != wire::TAG || message.length() != wire::SIZE {
                    Err(Status::Invalid)
                } else {
                    wire::validate_request(&body, epoch, domain)
                };
            let mut result = Status::Ok;
            let mut revision = 0;
            let mut value = [0; journal::VALUE];
            let mut length = None;
            match validation {
                Err(error) => result = error,
                Ok(_) if uncertain => result = Status::Unknown,
                Ok(request) => {
                    let object = match current.get(request.object) {
                        Ok(object) => object,
                        Err(_) => rt::fail(71),
                    };
                    if request.operation == GET {
                        revision = object.revision;
                        if object.present {
                            value = object.value;
                            length = Some(object.length);
                        } else {
                            result = Status::NotFound;
                        }
                    } else if matches!(request.operation, PUT | DELETE) {
                        let payload = &body[64..64 + request.length as usize];
                        match current.prepare(
                            request.object,
                            request.revision,
                            request.request_id,
                            request.operation,
                            payload,
                        ) {
                            Err(error) => {
                                result = status(error);
                                if result == Status::NotFound {
                                    revision = object.revision;
                                }
                            }
                            Ok(prepared) => {
                                // No in-memory commit or app ACK before BOTH real
                                // completions. On any uncertain result this
                                // instance closes all valid operations until a
                                // fresh process performs full recovery.
                                rt::put(192, 2);
                                let persisted =
                                    write_record(cap, current.sequence(), prepared.record())
                                        .and_then(|()| {
                                            rt::put(192, 3);
                                            flush(cap)
                                        });
                                if persisted.is_err() {
                                    uncertain = true;
                                    result = Status::Unknown;
                                    rt::put(232, Status::Unknown as i32 as i64 as u64);
                                    rt::put(192, 5);
                                } else {
                                    if current.commit(prepared).is_err() {
                                        rt::fail(72);
                                    }
                                    revision = current
                                        .get(request.object)
                                        .unwrap_or_else(|_| rt::fail(73))
                                        .revision;
                                    rt::put(128, current.sequence());
                                    rt::put(192, 4);
                                }
                            }
                        }
                    } else {
                        rt::fail(74);
                    }
                }
            }
            let response = wire::reply(
                header,
                epoch,
                revision,
                result,
                length.map(|count| &value[..count]),
            );
            let mut message = rt::Message::new();
            message.payload(wire::TAG, &response);
            rt::send(replies[domain], &message);
            if !uncertain {
                rt::put(192, 1);
            }
        }
        rt::yield_now();
    }
}
