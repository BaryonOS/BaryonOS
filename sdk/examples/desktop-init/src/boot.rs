//! Fixed GUI bootmanifest metadata. It never creates or discovers handles.
use nieuw_os_sdk_runtime::{Handle, ReceiveEndpoint, SendEndpoint};
pub const MAGIC: u64 = u64::from_le_bytes(*b"NOSDINI1");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
#[derive(Clone, Copy, Debug)]
pub struct Controller {
    pub receive: ReceiveEndpoint,
    pub reply: SendEndpoint,
    pub pid: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Bootstrap {
    pub generation: u64,
    pub registry: ReceiveEndpoint,
    pub app_control: SendEndpoint,
    pub compositor_control: SendEndpoint,
    pub controller: Option<Controller>,
    pub app_pid: u64,
    pub compositor_pid: u64,
    pub source_pid: u64,
    pub policy_pid: u64,
    pub surface_epoch: u64,
    pub policy_epoch: u64,
    pub native_display_epoch: u64,
    pub client_id: u64,
}
fn handle(v: u64) -> Result<Handle, Invalid> {
    Handle::from_raw(v).map_err(|_| Invalid)
}
impl Bootstrap {
    pub fn decode(words: &[u64; 128]) -> Result<Self, Invalid> {
        let at = |o: usize| words[o / 8];
        if at(32) != MAGIC
            || [40, 112, 120, 128, 136].iter().any(|o| at(*o) == 0)
            || [at(80), at(88), at(96), at(104)] != [4, 5, 6, 7]
            || words[160 / 8..].iter().any(|v| *v != 0)
        {
            return Err(Invalid);
        }
        let controller = if [at(72), at(144), at(152)] == [0; 3] {
            None
        } else {
            if at(152) <= 7 {
                return Err(Invalid);
            }
            Some(Controller {
                receive: ReceiveEndpoint::from_handle(handle(at(72))?),
                reply: SendEndpoint::from_handle(handle(at(144))?),
                pid: at(152),
            })
        };
        let registry = ReceiveEndpoint::from_handle(handle(at(48))?);
        let app_control = SendEndpoint::from_handle(handle(at(56))?);
        let compositor_control = SendEndpoint::from_handle(handle(at(64))?);
        let mut handles = [None; 6];
        handles[0] = Some(registry.handle());
        handles[1] = Some(app_control.handle());
        handles[2] = Some(compositor_control.handle());
        if let Some(c) = controller {
            handles[3] = Some(c.receive.handle());
            handles[4] = Some(c.reply.handle());
        }
        for (i, h) in handles.iter().enumerate() {
            if h.is_some() && handles[..i].contains(h) {
                return Err(Invalid);
            }
        }
        Ok(Self {
            generation: at(40),
            registry,
            app_control,
            compositor_control,
            controller,
            app_pid: at(80),
            compositor_pid: at(88),
            source_pid: at(96),
            policy_pid: at(104),
            surface_epoch: at(112),
            policy_epoch: at(120),
            native_display_epoch: at(128),
            client_id: at(136),
        })
    }
    pub fn handles(self) -> [Option<Handle>; 5] {
        [
            Some(self.registry.handle()),
            Some(self.app_control.handle()),
            Some(self.compositor_control.handle()),
            self.controller.map(|c| c.receive.handle()),
            self.controller.map(|c| c.reply.handle()),
        ]
    }
}
