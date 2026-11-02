//! Real capability-engine + v7 volume-service host harness. Ordered RAM sectors
//! are a test backend, not native durable disk or authenticated storage proof.
use capability_core::{Engine, Message, SEND};
use nieuw_os_catalog_client::Transport;
use nieuw_os_document_session::{
    volume::{Buffers, VolumeSession},
    EditMode,
};
use nieuw_os_materiaal::{controller::Controller, Scale, Viewport};
use nieuw_os_sdk_runtime::{
    Error as RuntimeError, Handle, Outgoing, ReceiveEndpoint, Received, SendEndpoint,
};
use std::{cell::RefCell, rc::Rc};
use volume_core::{
    client::{Client, Transport as CoreTransport},
    service::Service,
    table::{self, Table},
    upload, wire, Disk, Status, SECTOR, VOLUME_SECTORS,
};

pub const EPOCH: u64 = 17;
pub const INPUT_EPOCH: u64 = 31;
pub struct MemoryDisk {
    sectors: Vec<[u8; SECTOR]>,
    pub fail: bool,
}
impl Disk for MemoryDisk {
    fn read(&mut self, sector: u64, out: &mut [u8; SECTOR]) -> Result<(), Status> {
        *out = *self.sectors.get(sector as usize).ok_or(Status::Invalid)?;
        Ok(())
    }
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), Status> {
        if self.fail {
            return Err(Status::Io);
        }
        *self
            .sectors
            .get_mut(sector as usize)
            .ok_or(Status::Invalid)? = *bytes;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Status> {
        if self.fail {
            Err(Status::Io)
        } else {
            Ok(())
        }
    }
}
pub struct Host {
    pub engine: Engine,
    pub service: Service,
    pub disk: MemoryDisk,
    pub request_root: u64,
    request_receive: u64,
    reply_send: u64,
    app_send: u64,
    app_receive: u64,
    pub drop_commit: bool,
    pub wrong_peer: bool,
    pub requests: Vec<wire::Header>,
    pub closed: Vec<u64>,
}
impl Host {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut engine = Engine::new();
        for pid in 1..=3 {
            engine.register(pid).unwrap();
        }
        let request_root = engine.create_endpoint(2).unwrap();
        let request_receive = engine
            .derive(2, request_root, capability_core::RECV)
            .unwrap();
        let app_send = engine.bootstrap_delegate(2, request_root, 1, SEND).unwrap();
        let app_receive = engine.create_endpoint(1).unwrap();
        let reply_send = engine.bootstrap_delegate(1, app_receive, 2, SEND).unwrap();
        let mut sectors = vec![[0; SECTOR]; VOLUME_SECTORS as usize];
        for index in 0..volume_core::TABLE_SECTORS {
            sectors[index as usize] = Table::EMPTY.encode_sector(index);
        }
        Rc::new(RefCell::new(Self {
            engine,
            service: Service::new(EPOCH, Table::EMPTY, false),
            disk: MemoryDisk {
                sectors,
                fail: false,
            },
            request_root,
            request_receive,
            reply_send,
            app_send,
            app_receive,
            drop_commit: false,
            wrong_peer: false,
            requests: Vec::new(),
            closed: Vec::new(),
        }))
    }
    fn pump(&mut self) {
        let request = match self.engine.receive(2, self.request_receive) {
            Ok(m) => m,
            Err(capability_core::Error::Again) => return,
            Err(e) => panic!("receive {e:?}"),
        };
        assert_eq!(request.tag, upload::TAG);
        assert_eq!(request.cap_count, 0);
        assert_eq!(request.len, wire::SIZE as u32);
        let body: [u8; wire::SIZE] = request.data[..wire::SIZE].try_into().unwrap();
        assert_eq!(u16::from_le_bytes([body[0], body[1]]), upload::VERSION);
        let header = wire::header(&body);
        self.requests.push(header);
        let reply = self
            .service
            .handle_for(0, request.sender_pid, &body, &mut self.disk);
        if self.drop_commit && header.operation == wire::COMMIT {
            return;
        }
        let mut message = Message::new();
        message.tag = upload::TAG;
        message.len = wire::SIZE as u32;
        message.data[..wire::SIZE].copy_from_slice(&reply);
        self.engine.send(2, self.reply_send, &message).unwrap();
    }
    pub fn seed(&mut self, object: u64, bytes: &[u8]) {
        self.seed_named(object, b"Document", bytes);
    }
    pub fn seed_named(&mut self, object: u64, name: &[u8], bytes: &[u8]) {
        let mut client = Client::new(Direct(&mut self.service, &mut self.disk), EPOCH, 100_000);
        let revision = client.stat(object).map_or(0, |s| s.revision);
        client
            .write(object, revision, name, bytes.len(), |offset, out| {
                out.copy_from_slice(&bytes[offset..offset + out.len()])
            })
            .unwrap();
    }
    pub fn recover(&mut self) -> Table {
        let mut table = Table::EMPTY;
        table::recover(&mut self.disk, &mut table).unwrap();
        table
    }
    pub fn bytes(&mut self, object: u64) -> Vec<u8> {
        let mut client = Client::new(Direct(&mut self.service, &mut self.disk), EPOCH, 900_000);
        let stat = client.stat(object).unwrap();
        let mut bytes = vec![0; stat.length];
        client
            .read(object, stat.revision, stat.length, |offset, chunk| {
                bytes[offset..offset + chunk.len()].copy_from_slice(chunk)
            })
            .unwrap();
        bytes
    }
}
struct Direct<'a>(&'a mut Service, &'a mut MemoryDisk);
impl CoreTransport for Direct<'_> {
    fn call(&mut self, request: &[u8; wire::SIZE]) -> Result<[u8; wire::SIZE], Status> {
        Ok(self.0.handle(0, request, self.1))
    }
}
pub struct Port(Rc<RefCell<Host>>);
impl Transport for Port {
    fn send(&mut self, endpoint: SendEndpoint, message: &Outgoing) -> Result<(), RuntimeError> {
        self.0
            .borrow_mut()
            .engine
            .send(
                1,
                endpoint.handle().raw(),
                &Message::decode(message.as_bytes()).unwrap(),
            )
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
    fn receive(&mut self, endpoint: ReceiveEndpoint) -> Result<Received, RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.pump();
        let message = h
            .engine
            .receive(1, endpoint.handle().raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))?;
        let mut bytes = message.encode();
        if h.wrong_peer {
            bytes[..8].copy_from_slice(&3u64.to_le_bytes());
        }
        Received::from_bytes(bytes, message.len as u64)
    }
    fn yield_now(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }
    fn close(&mut self, handle: Handle) -> Result<(), RuntimeError> {
        let mut h = self.0.borrow_mut();
        h.closed.push(handle.raw());
        h.engine
            .close(1, handle.raw())
            .map_err(|e| RuntimeError::Kernel(e.code()))
    }
}
pub fn controller<'a>(
    host: &Rc<RefCell<Host>>,
    mode: EditMode,
    buffers: Buffers<'a>,
) -> Controller<'a, Port> {
    let h = host.borrow();
    let session = VolumeSession::new(
        Port(Rc::clone(host)),
        EPOCH,
        2,
        0,
        SendEndpoint::from_handle(Handle::from_raw(h.app_send).unwrap()),
        ReceiveEndpoint::from_handle(Handle::from_raw(h.app_receive).unwrap()),
        4,
    )
    .unwrap();
    Controller::new(
        session,
        mode,
        buffers,
        Viewport::new(800, 600, Scale::Percent100, 1).unwrap(),
        INPUT_EPOCH,
    )
    .unwrap()
}
