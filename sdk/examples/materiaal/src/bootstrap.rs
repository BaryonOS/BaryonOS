//! Narrow reader for the current kernel-supplied startup page, not authority
//! from a self-declared PID, epoch, name or handle integer.
use nieuw_os_materiaal::Scale;
use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, SendEndpoint};

#[derive(Clone, Copy, Debug)]
pub struct Bootstrap {
    pub document_epoch: u64,
    pub volume_pid: u64,
    pub self_pid: u64,
    pub init_pid: u64,
    pub document_send: SendEndpoint,
    pub document_receive: ReceiveEndpoint,
    pub registry_send: SendEndpoint,
    pub control_receive: ReceiveEndpoint,
    pub read_only: bool,
    pub surface_send: SendEndpoint,
    pub surface_receive: ReceiveEndpoint,
    pub compositor_pid: u64,
    pub surface_epoch: u64,
    pub input_send: SendEndpoint,
    pub input_receive: ReceiveEndpoint,
    pub policy_pid: u64,
    pub input_epoch: u64,
    pub input_client: u64,
    pub scale: Scale,
}
impl Bootstrap {
    pub fn read(word: impl Fn(u64) -> u64) -> Option<Self> {
        let raw = [
            word(80),
            word(72),
            word(88),
            word(96),
            word(544),
            word(552),
            word(576),
            word(584),
        ];
        for (i, &h) in raw.iter().enumerate() {
            Handle::from_raw(h).ok()?;
            if raw[..i].contains(&h) {
                return None;
            }
        }
        let pids = [word(168), word(176), word(184), word(560), word(592)];
        for (i, &pid) in pids.iter().enumerate() {
            if pid == 0
                || (pids[..i].contains(&pid) && !(word(656) == 1 && i == 4 && pid == pids[3]))
            {
                return None;
            }
        }
        if word(32) == 0
            || word(568) == 0
            || word(600) == 0
            || word(608) == 0
            || word(112) > 1
            || word(656) > 1
        {
            return None;
        }
        let send = |i| Some(SendEndpoint::from_handle(Handle::from_raw(raw[i]).ok()?));
        let recv = |i| Some(ReceiveEndpoint::from_handle(Handle::from_raw(raw[i]).ok()?));
        Some(Self {
            document_epoch: word(32),
            volume_pid: pids[0],
            self_pid: pids[1],
            init_pid: pids[2],
            document_send: send(0)?,
            document_receive: recv(1)?,
            registry_send: send(2)?,
            control_receive: recv(3)?,
            read_only: word(112) == 1,
            surface_send: send(4)?,
            surface_receive: recv(5)?,
            compositor_pid: pids[3],
            surface_epoch: word(568),
            input_send: send(6)?,
            input_receive: recv(7)?,
            policy_pid: pids[4],
            input_epoch: word(600),
            input_client: word(608),
            scale: match word(616) {
                1 => Scale::Percent100,
                2 => Scale::Percent200,
                _ => return None,
            },
        })
    }
}
