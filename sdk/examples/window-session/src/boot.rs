use nieuw_os_display_client::DisplaySource;
use nieuw_os_sdk_runtime::{Handle, Outgoing, ReceiveEndpoint, SendEndpoint};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy)]
pub struct App {
    pub pid: u64,
    pub surface_reply: SendEndpoint,
    pub input_reply: SendEndpoint,
    pub surface_epoch: u64,
    pub input_epoch: u64,
    pub client: u64,
}
#[derive(Clone, Copy)]
pub struct Bootstrap {
    pub generation: u64,
    pub display: DisplaySource,
    pub ingress: ReceiveEndpoint,
    pub focus: SendEndpoint,
    pub focus_reply: ReceiveEndpoint,
    pub policy_pid: u64,
    pub policy_epoch: u64,
    pub input: SendEndpoint,
    pub input_reply: ReceiveEndpoint,
    pub registry: SendEndpoint,
    pub init_pid: u64,
    pub self_pid: u64,
    pub scale: u32,
    pub apps: [App; 2],
}
pub const READY: u64 = u64::from_le_bytes(*b"IPDREAD1");
pub const CONTROL: u64 = 0x5359_5354_454d_0001;
fn handle(n: u64) -> Result<Handle, Invalid> {
    Handle::from_raw(n).map_err(|_| Invalid)
}
impl Bootstrap {
    pub fn ready(&self, native_epoch: u64) -> Result<Outgoing, Invalid> {
        if native_epoch == 0 {
            return Err(Invalid);
        }
        let mut b = [0u8; 32];
        b[..4].copy_from_slice(b"IPD1");
        b[4..6].copy_from_slice(&1u16.to_le_bytes());
        b[6..8].copy_from_slice(&3u16.to_le_bytes());
        b[8..16].copy_from_slice(&self.apps[0].surface_epoch.to_le_bytes());
        b[16..24].copy_from_slice(&self.generation.to_le_bytes());
        b[24..].copy_from_slice(&native_epoch.to_le_bytes());
        Outgoing::new(READY, &b).map_err(|_| Invalid)
    }
    pub fn decode(words: &[u64; 128]) -> Result<Self, Invalid> {
        let w = |at: usize| words[at / 8];
        if w(32) != u64::from_le_bytes(*b"WINCOMP1")
            || w(40) == 0
            || w(88) == 0
            || !matches!(w(136), 1 | 2)
            || words[144 / 8..160 / 8].iter().any(|v| *v != 0)
            || words[208 / 8..224 / 8].iter().any(|v| *v != 0)
            || words[272 / 8..].iter().any(|v| *v != 0)
        {
            return Err(Invalid);
        }
        let app = |at| {
            Ok(App {
                pid: w(at),
                surface_reply: SendEndpoint::from_handle(handle(w(at + 8))?),
                input_reply: SendEndpoint::from_handle(handle(w(at + 16))?),
                surface_epoch: w(at + 24),
                input_epoch: w(at + 32),
                client: w(at + 40),
            })
        };
        let b = Self {
            generation: w(40),
            display: DisplaySource::from_handle(handle(w(48))?),
            ingress: ReceiveEndpoint::from_handle(handle(w(56))?),
            focus: SendEndpoint::from_handle(handle(w(64))?),
            focus_reply: ReceiveEndpoint::from_handle(handle(w(72))?),
            policy_pid: w(80),
            policy_epoch: w(88),
            input: SendEndpoint::from_handle(handle(w(96))?),
            input_reply: ReceiveEndpoint::from_handle(handle(w(104))?),
            registry: SendEndpoint::from_handle(handle(w(112))?),
            init_pid: w(120),
            self_pid: w(128),
            scale: w(136) as u32,
            apps: [app(160)?, app(224)?],
        };
        let pids = [
            b.policy_pid,
            b.init_pid,
            b.self_pid,
            b.apps[0].pid,
            b.apps[1].pid,
        ];
        if pids.contains(&0)
            || pids.iter().enumerate().any(|(i, p)| pids[..i].contains(p))
            || b.apps
                .iter()
                .any(|a| a.surface_epoch == 0 || a.input_epoch == 0 || a.client == 0)
            || b.apps[0].client == b.apps[1].client
        {
            return Err(Invalid);
        }
        let handles = [
            w(48),
            w(56),
            w(64),
            w(72),
            w(96),
            w(104),
            w(112),
            w(168),
            w(176),
            w(232),
            w(240),
        ];
        if handles
            .iter()
            .enumerate()
            .any(|(i, h)| handles[..i].contains(h))
        {
            return Err(Invalid);
        }
        Ok(b)
    }
}
