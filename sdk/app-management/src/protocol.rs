#![forbid(unsafe_code)]

use crate::{PackageIdentity, IDENTITY_LEN, MAX_PACKAGE_LEN};
use nieuw_os_sdk_runtime::{Received, Rights, MAX_CAPS, MAX_PAYLOAD};

pub const VERSION: u16 = 1;
pub const TAG: u64 = 0x4e4f_5341_5050_0001;
pub const HEADER_LEN: usize = 24;
pub const MAX_GRANTS: usize = MAX_CAPS;
pub const GRANT_LEN: usize = 32;
pub const CHUNK_DATA_MAX: usize = MAX_PAYLOAD - HEADER_LEN - 8;
pub const APPENDIX: usize = 8;
pub const REPLY_HEADER_LEN: usize = 24;

const INSTALL_BEGIN: u16 = 1;
const INSTALL_CHUNK: u16 = 2;
const INSTALL_COMMIT: u16 = 3;
const UPDATE_BEGIN: u16 = 4;
const REMOVE: u16 = 5;
const LAUNCH: u16 = 6;
const CANCEL: u16 = 7;
const RESULT: u16 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    WrongTag,
    WrongSender,
    WrongEpoch,
    UnsupportedVersion,
    UnknownOperation,
    InvalidLength,
    InvalidIdentity,
    InvalidSource,
    InvalidPackageLength,
    InvalidRequestId,
    InvalidRights,
    CapabilityCount,
    StaleGrant,
    Replay,
    WrongTransaction,
    OutOfOrder,
    Overflow,
    Incomplete,
    CancelTooLate,
    InvalidReply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadAction {
    Install,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrantDescriptor {
    /// Opaque identifier interpreted only by the policy broker.
    pub scope_id: u64,
    pub rights: u64,
    pub epoch: u64,
}

const EMPTY_GRANT: GrantDescriptor = GrantDescriptor {
    scope_id: 0,
    rights: 0,
    epoch: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchRequest {
    pub identity: PackageIdentity,
    pub grants: [GrantDescriptor; MAX_GRANTS],
    pub grant_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request<'a> {
    Begin {
        action: UploadAction,
        package_length: u32,
        source_id: [u8; 32],
        source_generation: u32,
        package_sha256: [u8; 32],
    },
    Chunk {
        offset: u32,
        bytes: &'a [u8],
    },
    Commit,
    Remove {
        identity: PackageIdentity,
        expected_catalog_revision: u64,
    },
    Launch(LaunchRequest),
    Cancel {
        target_request_id: u64,
    },
    Result {
        target_request_id: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedRequest<'a> {
    pub sender: u64,
    pub service_epoch: u64,
    pub request_id: u64,
    pub request: Request<'a>,
}

/// Decode a message from the PID authorized for this operation. The sender
/// comes from `Received`, which is populated by the kernel, not the payload.
pub fn decode_request<'a>(
    message: &'a Received,
    expected_sender: u64,
    expected_epoch: u64,
) -> Result<DecodedRequest<'a>, ProtocolError> {
    if message.tag() != TAG {
        return Err(ProtocolError::WrongTag);
    }
    let sender = message.sender();
    if expected_sender == 0 || sender != expected_sender {
        return Err(ProtocolError::WrongSender);
    }
    let payload = message.payload();
    if payload.len() < HEADER_LEN {
        return Err(ProtocolError::InvalidLength);
    }
    if read_u16(payload, 0) != VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    if payload[4..8].iter().any(|byte| *byte != 0) {
        return Err(ProtocolError::InvalidLength);
    }
    let operation = read_u16(payload, 2);
    let request_id = read_u64(payload, 8);
    let service_epoch = read_u64(payload, 16);
    if request_id == 0 {
        return Err(ProtocolError::InvalidRequestId);
    }
    if expected_epoch == 0 || service_epoch != expected_epoch {
        return Err(ProtocolError::WrongEpoch);
    }
    let data = &payload[HEADER_LEN..];
    let request = match operation {
        INSTALL_BEGIN | UPDATE_BEGIN => {
            if !message_caps(message, 0) || data.len() != 80 {
                return Err(ProtocolError::InvalidLength);
            }
            let action = if operation == INSTALL_BEGIN {
                UploadAction::Install
            } else {
                UploadAction::Update
            };
            if data[1..8].iter().any(|byte| *byte != 0) {
                return Err(ProtocolError::InvalidLength);
            }
            let mut source_id = [0; 32];
            source_id.copy_from_slice(&data[8..40]);
            let source_generation = read_u32(data, 40);
            let package_length = read_u32(data, 44);
            let mut package_sha256 = [0; 32];
            package_sha256.copy_from_slice(&data[48..80]);
            if source_id.iter().all(|byte| *byte == 0) || source_generation == 0 {
                return Err(ProtocolError::InvalidSource);
            }
            if package_length == 0 || package_length as usize > MAX_PACKAGE_LEN {
                return Err(ProtocolError::InvalidPackageLength);
            }
            Request::Begin {
                action,
                package_length,
                source_id,
                source_generation,
                package_sha256,
            }
        }
        INSTALL_CHUNK => {
            if !message_caps(message, 0) || data.len() < 8 {
                return Err(ProtocolError::InvalidLength);
            }
            let offset = read_u32(data, 0);
            let length = read_u16(data, 4) as usize;
            if read_u16(data, 6) != 0
                || length == 0
                || length > CHUNK_DATA_MAX
                || data.len() != 8 + length
            {
                return Err(ProtocolError::InvalidLength);
            }
            offset
                .checked_add(length as u32)
                .ok_or(ProtocolError::Overflow)?;
            Request::Chunk {
                offset,
                bytes: &data[8..],
            }
        }
        INSTALL_COMMIT => {
            if !message_caps(message, 0) || !data.is_empty() {
                return Err(ProtocolError::InvalidLength);
            }
            Request::Commit
        }
        REMOVE => {
            if !message_caps(message, 0) || data.len() != IDENTITY_LEN + 16 {
                return Err(ProtocolError::InvalidLength);
            }
            let identity = PackageIdentity::decode(&data[..IDENTITY_LEN])?;
            let expected_catalog_revision = read_u64(data, IDENTITY_LEN);
            if expected_catalog_revision == 0
                || data[IDENTITY_LEN + 8..].iter().any(|byte| *byte != 0)
            {
                return Err(ProtocolError::InvalidLength);
            }
            Request::Remove {
                identity,
                expected_catalog_revision,
            }
        }
        LAUNCH => {
            if data.len() < IDENTITY_LEN + APPENDIX {
                return Err(ProtocolError::InvalidLength);
            }
            let identity = PackageIdentity::decode(&data[..IDENTITY_LEN])?;
            let grant_count = data[IDENTITY_LEN] as usize;
            if grant_count > MAX_GRANTS
                || data[IDENTITY_LEN + 1..IDENTITY_LEN + APPENDIX]
                    .iter()
                    .any(|byte| *byte != 0)
                || data.len() != IDENTITY_LEN + APPENDIX + grant_count * GRANT_LEN
                || !message_caps(message, grant_count)
            {
                return Err(ProtocolError::CapabilityCount);
            }
            let mut grants = [EMPTY_GRANT; MAX_GRANTS];
            for (index, grant) in grants.iter_mut().enumerate().take(grant_count) {
                let at = IDENTITY_LEN + APPENDIX + index * GRANT_LEN;
                let rights = read_u64(data, at);
                let epoch = read_u64(data, at + 8);
                let scope_id = read_u64(data, at + 16);
                if data[at + 24..at + 32].iter().any(|byte| *byte != 0)
                    || scope_id == 0
                    || epoch == 0
                    || rights == 0
                    || Rights::from_bits(rights).is_err()
                    || rights & Rights::RECV.bits() != 0
                {
                    return Err(ProtocolError::InvalidRights);
                }
                let attached = message.cap(index).ok_or(ProtocolError::CapabilityCount)?;
                if attached.rights.bits() != rights {
                    return Err(ProtocolError::InvalidRights);
                }
                *grant = GrantDescriptor {
                    scope_id,
                    rights,
                    epoch,
                };
            }
            Request::Launch(LaunchRequest {
                identity,
                grants,
                grant_count,
            })
        }
        CANCEL | RESULT => {
            if !message_caps(message, 0) || data.len() != 8 {
                return Err(ProtocolError::InvalidLength);
            }
            let target_request_id = read_u64(data, 0);
            if target_request_id == 0 || target_request_id == request_id {
                return Err(ProtocolError::InvalidRequestId);
            }
            if operation == CANCEL {
                Request::Cancel { target_request_id }
            } else {
                Request::Result { target_request_id }
            }
        }
        _ => return Err(ProtocolError::UnknownOperation),
    };
    Ok(DecodedRequest {
        sender,
        service_epoch,
        request_id,
        request,
    })
}

pub fn validate_grant_epochs(
    request: &LaunchRequest,
    current_epochs: &[u64; MAX_GRANTS],
) -> Result<(), ProtocolError> {
    if request.grant_count > MAX_GRANTS {
        return Err(ProtocolError::CapabilityCount);
    }
    for (index, grant) in request.grants.iter().enumerate().take(request.grant_count) {
        if grant.epoch != current_epochs[index] {
            return Err(ProtocolError::StaleGrant);
        }
    }
    Ok(())
}

fn message_caps(message: &Received, expected: usize) -> bool {
    message.cap_count() == expected
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ReplyStatus {
    Ok = 0,
    Invalid = 1,
    Unauthorized = 2,
    Stale = 3,
    Full = 4,
    Busy = 5,
    Unknown = 6,
    NotFound = 7,
    Failed = 8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OperationState {
    Pending = 1,
    Succeeded = 2,
    Failed = 3,
    Cancelled = 4,
    Unknown = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyData {
    None,
    UploadProgress {
        next_offset: u32,
    },
    PackageCommitted {
        identity: PackageIdentity,
        catalog_revision: u64,
    },
    Launched {
        launch_id: u64,
        identity: PackageIdentity,
    },
    Operation {
        target_request_id: u64,
        state: OperationState,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reply {
    pub operation: u16,
    pub request_id: u64,
    pub service_epoch: u64,
    pub status: ReplyStatus,
    pub data: ReplyData,
}

pub fn encode_reply(reply: &Reply) -> Result<nieuw_os_sdk_runtime::Outgoing, ProtocolError> {
    let mut payload = [0u8; MAX_PAYLOAD];
    payload[0..2].copy_from_slice(&VERSION.to_le_bytes());
    payload[2..4].copy_from_slice(&reply.operation.to_le_bytes());
    payload[4..6].copy_from_slice(&(reply.status as u16).to_le_bytes());
    payload[8..16].copy_from_slice(&reply.request_id.to_le_bytes());
    payload[16..24].copy_from_slice(&reply.service_epoch.to_le_bytes());
    let length = match reply.data {
        ReplyData::None => REPLY_HEADER_LEN,
        ReplyData::UploadProgress { next_offset } => {
            payload[24..28].copy_from_slice(&next_offset.to_le_bytes());
            28
        }
        ReplyData::PackageCommitted {
            identity,
            catalog_revision,
        } => {
            identity.encode(&mut payload[24..24 + IDENTITY_LEN]);
            payload[24 + IDENTITY_LEN..32 + IDENTITY_LEN]
                .copy_from_slice(&catalog_revision.to_le_bytes());
            REPLY_HEADER_LEN + IDENTITY_LEN + 8
        }
        ReplyData::Launched {
            launch_id,
            identity,
        } => {
            if launch_id == 0 {
                return Err(ProtocolError::InvalidReply);
            }
            payload[24..32].copy_from_slice(&launch_id.to_le_bytes());
            identity.encode(&mut payload[32..32 + IDENTITY_LEN]);
            REPLY_HEADER_LEN + 8 + IDENTITY_LEN
        }
        ReplyData::Operation {
            target_request_id,
            state,
        } => {
            if target_request_id == 0 || target_request_id == reply.request_id {
                return Err(ProtocolError::InvalidReply);
            }
            payload[24] = state as u8;
            payload[32..40].copy_from_slice(&target_request_id.to_le_bytes());
            40
        }
    };
    if reply.request_id == 0 || reply.service_epoch == 0 {
        return Err(ProtocolError::InvalidReply);
    }
    if reply.status != ReplyStatus::Ok && !matches!(reply.data, ReplyData::None) {
        return Err(ProtocolError::InvalidReply);
    }
    let outgoing = nieuw_os_sdk_runtime::Outgoing::new(TAG, &payload[..length])
        .map_err(|_| ProtocolError::InvalidLength)?;
    Ok(outgoing)
}

pub fn decode_reply(
    message: &Received,
    expected_sender: u64,
    expected_epoch: u64,
    expected_operation: u16,
    expected_request_id: u64,
) -> Result<Reply, ProtocolError> {
    if message.tag() != TAG {
        return Err(ProtocolError::WrongTag);
    }
    if expected_sender == 0 || message.sender() != expected_sender {
        return Err(ProtocolError::WrongSender);
    }
    if message.cap_count() != 0 {
        return Err(ProtocolError::CapabilityCount);
    }
    let payload = message.payload();
    if payload.len() < REPLY_HEADER_LEN {
        return Err(ProtocolError::InvalidLength);
    }
    if read_u16(payload, 0) != VERSION {
        return Err(ProtocolError::UnsupportedVersion);
    }
    let operation = read_u16(payload, 2);
    let status = match read_u16(payload, 4) {
        0 => ReplyStatus::Ok,
        1 => ReplyStatus::Invalid,
        2 => ReplyStatus::Unauthorized,
        3 => ReplyStatus::Stale,
        4 => ReplyStatus::Full,
        5 => ReplyStatus::Busy,
        6 => ReplyStatus::Unknown,
        7 => ReplyStatus::NotFound,
        8 => ReplyStatus::Failed,
        _ => return Err(ProtocolError::InvalidReply),
    };
    let request_id = read_u64(payload, 8);
    let service_epoch = read_u64(payload, 16);
    if payload[6..8].iter().any(|byte| *byte != 0)
        || operation != expected_operation
        || request_id != expected_request_id
        || request_id == 0
    {
        return Err(ProtocolError::InvalidReply);
    }
    if expected_epoch == 0 || service_epoch != expected_epoch {
        return Err(ProtocolError::WrongEpoch);
    }
    let data = &payload[REPLY_HEADER_LEN..];
    let decoded = if status != ReplyStatus::Ok {
        if !data.is_empty() {
            return Err(ProtocolError::InvalidReply);
        }
        ReplyData::None
    } else {
        match operation {
            INSTALL_BEGIN | UPDATE_BEGIN | INSTALL_CHUNK => {
                if data.len() != 4 {
                    return Err(ProtocolError::InvalidReply);
                }
                ReplyData::UploadProgress {
                    next_offset: read_u32(data, 0),
                }
            }
            INSTALL_COMMIT | REMOVE => {
                if data.len() != IDENTITY_LEN + 8 {
                    return Err(ProtocolError::InvalidReply);
                }
                let identity = PackageIdentity::decode(&data[..IDENTITY_LEN])?;
                let catalog_revision = read_u64(data, IDENTITY_LEN);
                if catalog_revision == 0 {
                    return Err(ProtocolError::InvalidReply);
                }
                ReplyData::PackageCommitted {
                    identity,
                    catalog_revision,
                }
            }
            LAUNCH => {
                if data.len() != 8 + IDENTITY_LEN {
                    return Err(ProtocolError::InvalidReply);
                }
                let launch_id = read_u64(data, 0);
                if launch_id == 0 {
                    return Err(ProtocolError::InvalidReply);
                }
                ReplyData::Launched {
                    launch_id,
                    identity: PackageIdentity::decode(&data[8..])?,
                }
            }
            CANCEL | RESULT => {
                if data.len() != 16 || data[1..8].iter().any(|byte| *byte != 0) {
                    return Err(ProtocolError::InvalidReply);
                }
                let state = match data[0] {
                    1 => OperationState::Pending,
                    2 => OperationState::Succeeded,
                    3 => OperationState::Failed,
                    4 => OperationState::Cancelled,
                    5 => OperationState::Unknown,
                    _ => return Err(ProtocolError::InvalidReply),
                };
                let target_request_id = read_u64(data, 8);
                if target_request_id == 0 || target_request_id == request_id {
                    return Err(ProtocolError::InvalidReply);
                }
                ReplyData::Operation {
                    target_request_id,
                    state,
                }
            }
            _ => return Err(ProtocolError::UnknownOperation),
        }
    };
    Ok(Reply {
        operation,
        request_id,
        service_epoch,
        status,
        data: decoded,
    })
}

fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nieuw_os_sdk_runtime::{Handle, Outgoing, MESSAGE_SIZE};
    use std::{vec, vec::Vec};

    fn payload(operation: u16, request_id: u64, epoch: u64, data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; HEADER_LEN + data.len()];
        bytes[0..2].copy_from_slice(&VERSION.to_le_bytes());
        bytes[2..4].copy_from_slice(&operation.to_le_bytes());
        bytes[8..16].copy_from_slice(&request_id.to_le_bytes());
        bytes[16..24].copy_from_slice(&epoch.to_le_bytes());
        bytes[HEADER_LEN..].copy_from_slice(data);
        bytes
    }

    fn received(sender: u64, payload: &[u8], capabilities: &[(u64, u64)]) -> Received {
        assert!(capabilities.len() <= MAX_CAPS);
        let mut bytes = [0; MESSAGE_SIZE];
        bytes[0..8].copy_from_slice(&sender.to_le_bytes());
        bytes[8..16].copy_from_slice(&TAG.to_le_bytes());
        bytes[16..20].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes[20..24].copy_from_slice(&(capabilities.len() as u32).to_le_bytes());
        for (index, (handle, rights)) in capabilities.iter().copied().enumerate() {
            bytes[32 + index * 16..40 + index * 16].copy_from_slice(&handle.to_le_bytes());
            bytes[40 + index * 16..48 + index * 16].copy_from_slice(&rights.to_le_bytes());
        }
        bytes[64..64 + payload.len()].copy_from_slice(payload);
        Received::from_bytes(bytes, payload.len() as u64).unwrap()
    }

    fn identity_bytes() -> [u8; IDENTITY_LEN] {
        let mut identity = [0; IDENTITY_LEN];
        identity[..15].copy_from_slice(b"com.example.app");
        identity[64..96].fill(1);
        identity[96..100].copy_from_slice(&1u32.to_le_bytes());
        identity[100..108].copy_from_slice(&1u64.to_le_bytes());
        identity[108..112].copy_from_slice(&4u32.to_le_bytes());
        identity[112..144].fill(2);
        identity
    }

    #[test]
    fn begin_is_bound_to_sender_epoch_and_package_digest() {
        let mut data = [0; 80];
        data[8..40].fill(3);
        data[40..44].copy_from_slice(&7u32.to_le_bytes());
        data[44..48].copy_from_slice(&((app_package_core::MANIFEST_LEN + 4) as u32).to_le_bytes());
        data[48..80].fill(9);
        let bytes = payload(INSTALL_BEGIN, 15, 6, &data);
        let message = received(44, &bytes, &[]);
        let decoded = decode_request(&message, 44, 6).unwrap();
        assert_eq!(decoded.request_id, 15);
        assert_eq!(decoded.service_epoch, 6);
        assert!(matches!(
            decoded.request,
            Request::Begin {
                action: UploadAction::Install,
                package_length,
                source_generation: 7,
                package_sha256,
                source_id,
            } if package_length as usize == app_package_core::MANIFEST_LEN + 4
                && package_sha256 == [9; 32]
                && source_id == [3; 32]
        ));
        assert_eq!(
            decode_request(&message, 45, 6),
            Err(ProtocolError::WrongSender)
        );
        assert_eq!(
            decode_request(&message, 44, 7),
            Err(ProtocolError::WrongEpoch)
        );
    }

    #[test]
    fn malformed_begin_chunk_and_remove_requests_fail_closed() {
        let mut begin_data = [0; 80];
        begin_data[8..40].fill(1);
        begin_data[40..44].copy_from_slice(&1u32.to_le_bytes());
        begin_data[44..48].copy_from_slice(&((MAX_PACKAGE_LEN + 1) as u32).to_le_bytes());
        let begin = received(1, &payload(INSTALL_BEGIN, 1, 1, &begin_data), &[]);
        assert_eq!(
            decode_request(&begin, 1, 1),
            Err(ProtocolError::InvalidPackageLength)
        );

        let mut chunk_data = [0; 10];
        chunk_data[..4].copy_from_slice(&u32::MAX.to_le_bytes());
        chunk_data[4..6].copy_from_slice(&2u16.to_le_bytes());
        let chunk = received(1, &payload(INSTALL_CHUNK, 2, 1, &chunk_data), &[]);
        assert_eq!(decode_request(&chunk, 1, 1), Err(ProtocolError::Overflow));

        let mut remove_data = [0; IDENTITY_LEN + 16];
        remove_data[..IDENTITY_LEN].copy_from_slice(&identity_bytes());
        remove_data[IDENTITY_LEN..IDENTITY_LEN + 8].copy_from_slice(&1u64.to_le_bytes());
        remove_data[IDENTITY_LEN + 8] = 1;
        let remove = received(1, &payload(REMOVE, 3, 1, &remove_data), &[]);
        assert_eq!(
            decode_request(&remove, 1, 1),
            Err(ProtocolError::InvalidLength)
        );
    }

    #[test]
    fn launch_defaults_to_no_grants_and_validates_capability_binding() {
        let mut no_grants = vec![0; IDENTITY_LEN + APPENDIX];
        no_grants[..IDENTITY_LEN].copy_from_slice(&identity_bytes());
        let message = received(7, &payload(LAUNCH, 8, 3, &no_grants), &[]);
        let decoded = decode_request(&message, 7, 3).unwrap();
        let Request::Launch(launch) = decoded.request else {
            panic!("expected launch");
        };
        assert_eq!(launch.grant_count, 0);
        assert_eq!(
            launch.grants, [EMPTY_GRANT; MAX_GRANTS],
            "manifest metadata does not imply grants"
        );

        let mut one_grant = vec![0; IDENTITY_LEN + APPENDIX + GRANT_LEN];
        one_grant[..IDENTITY_LEN].copy_from_slice(&identity_bytes());
        one_grant[IDENTITY_LEN] = 1;
        let grant = IDENTITY_LEN + APPENDIX;
        one_grant[grant..grant + 8].copy_from_slice(&Rights::SEND.bits().to_le_bytes());
        one_grant[grant + 8..grant + 16].copy_from_slice(&55u64.to_le_bytes());
        one_grant[grant + 16..grant + 24].copy_from_slice(&19u64.to_le_bytes());
        let capability = Handle::from_raw(0x1_0000_0001).unwrap();
        let outgoing = Outgoing::new(TAG, &payload(LAUNCH, 9, 3, &one_grant)).unwrap();
        let mut outgoing = outgoing;
        outgoing
            .push_cap(nieuw_os_sdk_runtime::Capability {
                handle: capability,
                rights: Rights::SEND,
            })
            .unwrap();
        let mut bytes = *outgoing.as_bytes();
        bytes[0..8].copy_from_slice(&7u64.to_le_bytes());
        let message = Received::from_bytes(bytes, (HEADER_LEN + one_grant.len()) as u64).unwrap();
        let decoded = decode_request(&message, 7, 3).unwrap();
        let Request::Launch(launch) = decoded.request else {
            panic!("expected launch");
        };
        assert_eq!(launch.grant_count, 1);
        assert_eq!(validate_grant_epochs(&launch, &[55, 0]), Ok(()));
        assert_eq!(
            validate_grant_epochs(&launch, &[56, 0]),
            Err(ProtocolError::StaleGrant)
        );

        one_grant[grant..grant + 8].copy_from_slice(&Rights::SEND.bits().to_le_bytes());
        let mismatch = received(
            7,
            &payload(LAUNCH, 10, 3, &one_grant),
            &[(0x1_0000_0001, Rights::BLOCK_WRITE.bits())],
        );
        assert_eq!(
            decode_request(&mismatch, 7, 3),
            Err(ProtocolError::InvalidRights)
        );
        assert_eq!(
            Rights::from_bits(Rights::RECV.bits()),
            Ok(Rights::RECV),
            "runtime validates representation; transfer policy rejects RECV"
        );
        let receive_cap = nieuw_os_sdk_runtime::Capability {
            handle: capability,
            rights: Rights::RECV,
        };
        assert_eq!(
            Outgoing::new(TAG, &[]).unwrap().push_cap(receive_cap),
            Err(nieuw_os_sdk_runtime::Error::InvalidRights)
        );
    }

    #[test]
    fn request_replay_and_reply_correlation_are_checked() {
        let result = received(7, &payload(RESULT, 10, 3, &11u64.to_le_bytes()), &[]);
        assert_eq!(
            decode_request(&result, 7, 3).unwrap().request,
            Request::Result {
                target_request_id: 11
            }
        );
        let replay = received(7, &payload(RESULT, 11, 3, &11u64.to_le_bytes()), &[]);
        assert_eq!(
            decode_request(&replay, 7, 3),
            Err(ProtocolError::InvalidRequestId)
        );

        let reply = Reply {
            operation: CANCEL,
            request_id: 12,
            service_epoch: 3,
            status: ReplyStatus::Ok,
            data: ReplyData::Operation {
                target_request_id: 11,
                state: OperationState::Cancelled,
            },
        };
        let outgoing = encode_reply(&reply).unwrap();
        let mut bytes = *outgoing.as_bytes();
        bytes[0..8].copy_from_slice(&77u64.to_le_bytes());
        let message = Received::from_bytes(bytes, 40).unwrap();
        assert_eq!(decode_reply(&message, 77, 3, CANCEL, 12).unwrap(), reply);
        assert_eq!(
            decode_reply(&message, 77, 4, CANCEL, 12),
            Err(ProtocolError::WrongEpoch)
        );
        assert_eq!(
            decode_reply(&message, 77, 3, CANCEL, 13),
            Err(ProtocolError::InvalidReply)
        );
    }
}
