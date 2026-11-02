//! Opt-in native test fixture data, never a product IPC/bootstrap ABI.
pub const CASE: u64 = 112;
pub const RAW: u64 = 0;
pub const SDK_CANCEL: u64 = 1;
pub const SDK_UNCERTAIN: u64 = 2;
pub const WRONG_PID: u64 = 3;
pub const SEND: u64 = 48;
pub const RECEIVE: u64 = 56;
pub const KEY_PID: u64 = 160;
pub const OWN_PID: u64 = 176;
pub const RESULT: u64 = 40;
pub const STEP: u64 = 240;
pub const SUCCESSES: u64 = 400;
pub const BYTES: u64 = 408;
pub const VARIABLES: u64 = 416;
pub const LEGACY_DENIED: u64 = 424;
pub const INVALID_DENIED: u64 = 432;
pub const QUOTA_DENIED: u64 = 440;
pub const CLOCK_READS: u64 = 448;
pub const ENQUEUED: u64 = 456;
pub const REBINDS: u64 = 464;
pub const STALE_DENIED: u64 = 472;
pub const REPLAY_DENIED: u64 = 480;
pub const REVOKED_DENIED: u64 = 488;
pub const SDK_OUTPUT_BYTES: u64 = 496;
pub const SDK_SEND_CALLS: u64 = 504;
pub const SDK_CLOSES: u64 = 512;
pub const SDK_CANCELLED: u64 = 520;
pub const SDK_UNKNOWN: u64 = 528;
pub const NO_RETRY: u64 = 536;
pub const PID_DENIED: u64 = 544;
pub const START_TICKS: u64 = 552;
pub const END_TICKS: u64 = 560;
pub const REPLIES: u64 = 568;
pub const CAP_DENIED: u64 = 576;
pub const SDK_POSITIVES: u64 = 584;
pub const COUNTERS: [u64; 24] = [
    SUCCESSES,
    BYTES,
    VARIABLES,
    LEGACY_DENIED,
    INVALID_DENIED,
    QUOTA_DENIED,
    CLOCK_READS,
    ENQUEUED,
    REBINDS,
    STALE_DENIED,
    REPLAY_DENIED,
    REVOKED_DENIED,
    SDK_OUTPUT_BYTES,
    SDK_SEND_CALLS,
    SDK_CLOSES,
    SDK_CANCELLED,
    SDK_UNKNOWN,
    NO_RETRY,
    PID_DENIED,
    START_TICKS,
    END_TICKS,
    REPLIES,
    CAP_DENIED,
    SDK_POSITIVES,
];
pub const REQUEST_PHASE: u64 = 640;
pub const ACK_PHASE: u64 = 648;
pub const NEW_SEND: u64 = 656;
pub const REBIND: u64 = 1;
pub const CLOSED_REPLY: u64 = 2;
pub const REVOKE: u64 = 3;
pub const SETTLE: u64 = 4;
pub const WRONG_PID_CONSUMED: u64 = 5;
pub const REQUEST_TICKS: u64 = 1000;
pub const WINDOW_TICKS: u64 = 100;
pub const NO_REPLY_TICKS: u64 = 10;
// Private RAW-failure metadata only. Never publish entropy or random bytes.
pub const DIAG_REQUEST: u64 = 664;
pub const DIAG_EXPECTED_COUNT: u64 = 672;
pub const DIAG_REQUEST_COUNT: u64 = 680;
pub const DIAG_STATUS_I32_BITS: u64 = 688;
pub const DIAG_REPLY_BODY_LENGTH: u64 = 696;
pub const DIAG_REPLY_PAYLOAD_LENGTH: u64 = 704;
pub const DIAG_BURST_START: u64 = 712;
pub const DIAG_BURST_END: u64 = 720;
pub const DIAG_FAILURE_TICK: u64 = 728;
pub const RAW_DIAGNOSTICS: [(&[u8], u64); 9] = [
    (b"SYSTEM:RANDOM_DIAG_REQUEST:", DIAG_REQUEST),
    (b"SYSTEM:RANDOM_DIAG_EXPECTED_COUNT:", DIAG_EXPECTED_COUNT),
    (b"SYSTEM:RANDOM_DIAG_REQUEST_COUNT:", DIAG_REQUEST_COUNT),
    (b"SYSTEM:RANDOM_DIAG_STATUS_I32_BITS:", DIAG_STATUS_I32_BITS),
    (
        b"SYSTEM:RANDOM_DIAG_REPLY_BODY_LENGTH:",
        DIAG_REPLY_BODY_LENGTH,
    ),
    (
        b"SYSTEM:RANDOM_DIAG_REPLY_PAYLOAD_LENGTH:",
        DIAG_REPLY_PAYLOAD_LENGTH,
    ),
    (b"SYSTEM:RANDOM_DIAG_BURST_START:", DIAG_BURST_START),
    (b"SYSTEM:RANDOM_DIAG_BURST_END:", DIAG_BURST_END),
    (b"SYSTEM:RANDOM_DIAG_FAILURE_TICK:", DIAG_FAILURE_TICK),
];
pub const SERVICE_WORDS: [u64; 12] = [272, 280, 288, 296, 304, 312, 320, 328, 336, 344, 352, 360];
pub const APP_MAGIC_AT: u64 = 624;
pub const APP_MAGIC: u64 = u64::from_le_bytes(*b"NOSRNGT1");
pub const APP_RELEASE: u64 = 632;
pub const APP_STATUS_ACK: u64 = 640;
pub const APP_STATUS_BEGIN: u64 = 648;
pub const APP_STATUS_END: u64 = 656;
