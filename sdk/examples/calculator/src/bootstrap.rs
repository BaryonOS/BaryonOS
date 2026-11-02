//! The six existing granted handles. Startup integers describe grants; only
//! the kernel and service ingress actually confer or validate authority.
use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, SendEndpoint};
#[derive(Clone, Copy, Debug)]
pub struct Bootstrap {
    pub registry: SendEndpoint,
    pub control: ReceiveEndpoint,
    pub surface_send: SendEndpoint,
    pub surface_receive: ReceiveEndpoint,
    pub input_send: SendEndpoint,
    pub input_receive: ReceiveEndpoint,
    pub self_pid: u64,
    pub init_pid: u64,
    pub compositor_pid: u64,
    pub policy_pid: u64,
    pub surface_epoch: u64,
    pub input_epoch: u64,
    pub input_client: u64,
    pub scale: u32,
}
impl Bootstrap {
    /// Before GO no typed owner has been constructed. Close every app-service
    /// leaf even if one close fails; registry/control remain for DONE/cleanup.
    pub fn close_unstarted(
        &self,
        mut close: impl FnMut(Handle) -> Result<(), nieuw_os_sdk_runtime::Error>,
    ) -> Result<(), nieuw_os_sdk_runtime::Error> {
        let mut error = None;
        for handle in [
            self.input_send.handle(),
            self.input_receive.handle(),
            self.surface_send.handle(),
            self.surface_receive.handle(),
        ] {
            if let Err(failure) = close(handle) {
                error.get_or_insert(failure);
            }
        }
        error.map_or(Ok(()), Err)
    }

    pub fn read(word: impl Fn(u64) -> u64) -> Option<Self> {
        let raw = [
            word(88),
            word(96),
            word(544),
            word(552),
            word(576),
            word(584),
        ];
        for (i, h) in raw.iter().enumerate() {
            Handle::from_raw(*h).ok()?;
            if raw[..i].contains(h) {
                return None;
            }
        }
        let pids = [word(176), word(184), word(560), word(592)];
        for (i, pid) in pids.iter().enumerate() {
            if *pid == 0
                || (pids[..i].contains(pid) && !(word(656) == 1 && i == 3 && *pid == pids[2]))
            {
                return None;
            }
        }
        // This app has no volume/document/key channel. Reject accidental legacy
        // document grants in this bootstrap rather than silently retaining them.
        if [word(48), word(56), word(64), word(72), word(80), word(168)]
            .iter()
            .any(|n| *n != 0)
            || word(656) > 1
            || word(568) == 0
            || word(600) == 0
            || word(608) == 0
            || !matches!(word(616), 1 | 2)
        {
            return None;
        }
        let send = |i| Some(SendEndpoint::from_handle(Handle::from_raw(raw[i]).ok()?));
        let recv = |i| Some(ReceiveEndpoint::from_handle(Handle::from_raw(raw[i]).ok()?));
        Some(Self {
            registry: send(0)?,
            control: recv(1)?,
            surface_send: send(2)?,
            surface_receive: recv(3)?,
            input_send: send(4)?,
            input_receive: recv(5)?,
            self_pid: pids[0],
            init_pid: pids[1],
            compositor_pid: pids[2],
            policy_pid: pids[3],
            surface_epoch: word(568),
            input_epoch: word(600),
            input_client: word(608),
            scale: word(616) as u32,
        })
    }
}
