//! Private fixture startup/evidence page, not an application bootstrap ABI.
pub const SOURCE: u64 = 48;
pub const WRONG_TYPE: u64 = 56;
pub const PEER_DISPLAY: u64 = 64;
pub const PEER_INPUT: u64 = 72;
pub const GENERATION: u64 = 80;
pub const CONTROL: u64 = 88;
pub const OLD_SOURCE: u64 = 96;
pub const FAILURE: u64 = 240;
pub const DONE: u64 = 400;
pub const BLITS: u64 = 408;
pub const DISPLAY_DENIED: u64 = 416;
pub const RESETS: u64 = 424;
pub const READY: u64 = 432;
pub const SAMPLES: u64 = 440;
pub const POINTER_DENIED: u64 = 448;
pub const CAP_DENIED: u64 = 456;
pub const UNCHANGED: u64 = 464;
pub const EPOCH: u64 = 472;
pub const TICKET: u64 = 480;
pub const WIDTH: u64 = 488;
pub const HEIGHT: u64 = 496;
pub const STAGE: u64 = 504;
pub const LOG: u64 = 512;
pub const CLOSED: u64 = 584;
pub const PENDING: u64 = 592;
pub const RESET_OPERATIONS: u64 = 600;
pub const RESET_DISCARDED: u64 = 608;
pub const OLD_DENIED: u64 = 616;
pub const LOG_SEQUENCE: u64 = 640;
pub const INPUT_BYTES: [u8; 8] = [0x1e, 0x9e, 0x09, 0x07, 0x03, 0x08, 0, 0];
pub const TILE_WIDTH: usize = 32;
pub const TILE_HEIGHT: usize = 8;
pub const TILE_BYTES: usize = TILE_WIDTH * TILE_HEIGHT * 4;
pub const HEAP_END: u64 = 0x60010000;
pub const BAD_POINTER: u64 = HEAP_END - 16;
pub const PARTIAL_PIXELS: u64 = HEAP_END - 320;
pub const SENTINEL: u8 = 0xa5;
// DERIVE/TRANSFER/REVOKE are outside these object types' supported right sets.
// The existing engine returns Invalid, before testing possession of the bit.
pub const SOURCE_RESTRICTED_ERROR: i64 = -1;

pub const fn tile_position(generation: u64) -> (u32, u32) {
    if generation == 1 {
        (64, 64)
    } else {
        (112, 80)
    }
}
/// Each pixel is different; the host independently compares its QMP screenshot.
pub const fn pixel(generation: u64, x: usize, y: usize) -> u32 {
    ((0x20 + generation as u32 * 0x20 + x as u32 * 3) << 16)
        | ((0x30 + y as u32 * 0x13) << 8)
        | (0x70 + x as u32 + y as u32)
}
