//! IPD1 fixture bootstrap. All words are descriptive private process data;
//! numeric handles cannot create authority and are checked by real SDK calls.
use crate::{
    policy::{Client, Replies},
    source,
};
use nieuw_os_display_client::InputSource;
use nieuw_os_input_core::{Actor, ClientBinding, Configuration, MAX_CLIENTS};
use nieuw_os_sdk_runtime::{Handle, Outgoing, ReceiveEndpoint, SendEndpoint};

pub const SOURCE_MAGIC: u64 = u64::from_le_bytes(*b"NOSISRC1");
pub const POLICY_MAGIC: u64 = u64::from_le_bytes(*b"NOSIPOL1");
pub const READY_TAG: u64 = u64::from_le_bytes(*b"IPDREAD1");
pub const WORDS: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
fn handle(raw: u64) -> Result<Handle, Invalid> {
    Handle::from_raw(raw).map_err(|_| Invalid)
}
fn send(raw: u64) -> Result<SendEndpoint, Invalid> {
    Ok(SendEndpoint::from_handle(handle(raw)?))
}
fn receive(raw: u64) -> Result<ReceiveEndpoint, Invalid> {
    Ok(ReceiveEndpoint::from_handle(handle(raw)?))
}
fn value(words: &[u64; WORDS], offset: usize) -> u64 {
    words[offset / 8]
}
pub struct SourceBoot {
    pub configuration: source::Configuration,
    pub input: InputSource,
    pub registry: SendEndpoint,
    pub init_pid: u64,
    pub generation: u64,
}
impl SourceBoot {
    pub fn decode(words: &[u64; WORDS]) -> Result<Self, Invalid> {
        if value(words, 32) != SOURCE_MAGIC
            || value(words, 40) == 0
            || value(words, 96) == 0
            || value(words, 96) == value(words, 72)
            || words[104 / 8..].iter().any(|x| *x != 0)
        {
            return Err(Invalid);
        }
        let configuration = source::Configuration {
            epoch: value(words, 80),
            policy_pid: value(words, 72),
            request: send(value(words, 56))?,
            response: receive(value(words, 64))?,
        };
        if configuration.epoch == 0 || configuration.policy_pid == 0 {
            return Err(Invalid);
        }
        Ok(Self {
            configuration,
            input: InputSource::from_handle(handle(value(words, 48))?),
            registry: send(value(words, 88))?,
            init_pid: value(words, 96),
            generation: value(words, 40),
        })
    }
}
pub struct PolicyBoot {
    pub configuration: Configuration,
    pub replies: Replies,
    pub clients: [Client; MAX_CLIENTS],
    pub client_count: usize,
    pub registry: SendEndpoint,
    pub generation: u64,
}
impl PolicyBoot {
    pub fn decode(words: &[u64; WORDS]) -> Result<Self, Invalid> {
        let count = usize::try_from(value(words, 160)).map_err(|_| Invalid)?;
        if value(words, 32) != POLICY_MAGIC
            || value(words, 40) == 0
            || !(1..=MAX_CLIENTS).contains(&count)
            || words[(168 + 24 * count) / 8..].iter().any(|x| *x != 0)
        {
            return Err(Invalid);
        }
        let configuration = Configuration {
            epoch: value(words, 104),
            source: Actor {
                endpoint: receive(value(words, 48))?,
                pid: value(words, 80),
            },
            compositor: Actor {
                endpoint: receive(value(words, 56))?,
                pid: value(words, 88),
            },
            supervisor: Actor {
                endpoint: receive(value(words, 64))?,
                pid: value(words, 96),
            },
            applications: receive(value(words, 72))?,
            width: u32::try_from(value(words, 112)).map_err(|_| Invalid)?,
            height: u32::try_from(value(words, 120)).map_err(|_| Invalid)?,
        };
        let client = |index: usize| -> Result<Client, Invalid> {
            let base = 168 + index * 24;
            Ok(Client {
                binding: ClientBinding {
                    id: value(words, base),
                    pid: value(words, base + 8),
                },
                reply: send(value(words, base + 16))?,
            })
        };
        let mut clients = [client(0)?; MAX_CLIENTS];
        for (index, entry) in clients.iter_mut().enumerate().take(count) {
            *entry = client(index)?;
        }
        Ok(Self {
            configuration,
            replies: Replies {
                source: send(value(words, 128))?,
                compositor: send(value(words, 136))?,
                supervisor: send(value(words, 144))?,
            },
            clients,
            client_count: count,
            registry: send(value(words, 152))?,
            generation: value(words, 40),
        })
    }
}
pub fn ready(role: u16, epoch: u64, generation: u64, raw_epoch: u64) -> Result<Outgoing, Invalid> {
    if epoch == 0
        || generation == 0
        || !matches!(role, 1 | 2)
        || (role == 1 && raw_epoch == 0)
        || (role == 2 && raw_epoch != 0)
    {
        return Err(Invalid);
    }
    let mut bytes = [0u8; 32];
    bytes[..4].copy_from_slice(b"IPD1");
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    bytes[6..8].copy_from_slice(&role.to_le_bytes());
    bytes[8..16].copy_from_slice(&epoch.to_le_bytes());
    bytes[16..24].copy_from_slice(&generation.to_le_bytes());
    bytes[24..].copy_from_slice(&raw_epoch.to_le_bytes());
    Outgoing::new(READY_TAG, &bytes).map_err(|_| Invalid)
}
