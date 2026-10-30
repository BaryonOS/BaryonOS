use nieuw_os_display_client::DisplaySource;
use nieuw_os_sdk_runtime::{Handle, Outgoing, ReceiveEndpoint, SendEndpoint};

pub const MAGIC: u64 = u64::from_le_bytes(*b"NOSCOMP1");
pub const READY_TAG: u64 = u64::from_le_bytes(*b"IPDREAD1");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy, Debug)]
pub struct Bootstrap {
    pub generation: u64,
    pub display: DisplaySource,
    pub surface: ReceiveEndpoint,
    pub app_reply: SendEndpoint,
    pub app_pid: u64,
    pub surface_epoch: u64,
    pub focus_send: SendEndpoint,
    pub focus_receive: ReceiveEndpoint,
    pub policy_pid: u64,
    pub policy_epoch: u64,
    pub control: ReceiveEndpoint,
    pub registry: SendEndpoint,
    pub init_pid: u64,
    pub client_id: u64,
}
fn handle(value: u64) -> Result<Handle, Invalid> {
    Handle::from_raw(value).map_err(|_| Invalid)
}
impl Bootstrap {
    pub fn decode(words: &[u64; 128]) -> Result<Self, Invalid> {
        let at = |offset: usize| words[offset / 8];
        if at(32) != MAGIC
            || [40, 72, 80, 104, 112, 136, 144]
                .iter()
                .any(|offset| at(*offset) == 0)
            || at(72) == at(104)
            || at(72) == at(136)
            || at(104) == at(136)
            || words[152 / 8..].iter().any(|x| *x != 0)
        {
            return Err(Invalid);
        }
        let result = Self {
            generation: at(40),
            display: DisplaySource::from_handle(handle(at(48))?),
            surface: ReceiveEndpoint::from_handle(handle(at(56))?),
            app_reply: SendEndpoint::from_handle(handle(at(64))?),
            app_pid: at(72),
            surface_epoch: at(80),
            focus_send: SendEndpoint::from_handle(handle(at(88))?),
            focus_receive: ReceiveEndpoint::from_handle(handle(at(96))?),
            policy_pid: at(104),
            policy_epoch: at(112),
            control: ReceiveEndpoint::from_handle(handle(at(120))?),
            registry: SendEndpoint::from_handle(handle(at(128))?),
            init_pid: at(136),
            client_id: at(144),
        };
        let receives = [result.surface, result.focus_receive, result.control];
        if receives
            .iter()
            .enumerate()
            .any(|(i, value)| receives[..i].contains(value))
        {
            return Err(Invalid);
        }
        Ok(result)
    }
    pub fn ready(&self, native_epoch: u64) -> Result<Outgoing, Invalid> {
        if native_epoch == 0 {
            return Err(Invalid);
        }
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(b"IPD1");
        bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
        bytes[6..8].copy_from_slice(&3u16.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.surface_epoch.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.generation.to_le_bytes());
        bytes[24..].copy_from_slice(&native_epoch.to_le_bytes());
        Outgoing::new(READY_TAG, &bytes).map_err(|_| Invalid)
    }
    pub fn handles(&self) -> [Handle; 7] {
        [
            self.display.handle(),
            self.surface.handle(),
            self.app_reply.handle(),
            self.focus_send.handle(),
            self.focus_receive.handle(),
            self.control.handle(),
            self.registry.handle(),
        ]
    }
}
