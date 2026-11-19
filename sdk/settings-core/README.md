# Settings core (standalone host model)

This allocator-free `no_std` crate forbids unsafe code. It provides typed,
versioned appearance/accessibility settings and A/B storage through a media
trait. It has no UI, native service, account/language settings, permission or
security-policy fields. It does not grant access to files, users or devices.

The standalone `[workspace]` and lockfile permit useful independent checks while
shared SDK workspace membership is pending the integration owner's changes.
This is a complete core slice, not completion of U05-T1 or native U05 acceptance.
No shared SDK manifest, lockfile or UI code is modified.

## Typed values and acknowledgement

Schema 1 contains `Theme::{System, Light, Dark}`, integer text scale from 75 to
300 percent inclusive, and a reduced-motion boolean. `Settings::new` validates
the scale; all fields are private. No implicit/default settings are loaded.
This is a bounded storage profile, not a UI/accessibility acceptance decision.

`SettingsStore::new` has no usable current state. Explicit `initialize(media,
settings)` first reads both complete slots and permits initialization only when
all 192 bytes are zero. Nonblank/corrupt storage is never silently reformatted.
A failed partial initialization may need separate owner-directed repair;
this core provides no destructive reset or migration operation.

`recover` reads both slots and selects the highest valid nonzero generation.
A bad checksum or invalid known-schema record may fall back to the other valid
slot, as in `sdk/app-management/src/journal.rs`. Both-corrupt storage and equal
valid generations are rejected. An I/O error aborts recovery; it cannot make an
unreadable newer slot look like an older current value. A checksum-valid record
with the recognized magic but an unknown schema version aborts recovery even
when the other slot is valid. No downgrade/migration is attempted.

`set` commits the supplied validated bundle atomically in this host model;
even an unchanged value gets a new generation. The record includes the previous
committed bundle. `revert` restores that bundle through a **new** commit and
retains the former current bundle as its previous value. Thus one-level revert
survives reload and repeated revert toggles the last two committed values;
it never returns an old generation acknowledgement. No previous value returns
`NoPrevious`. Generation increment is checked and never wraps.

For initialize/set/revert, the order is:

1. Retire in-memory current state before attempting the operation.
2. Write the complete canonical record to the inactive slot (slot 0 on initialize).
3. Flush, then read back that slot and compare every byte with the intended record.
4. Only on complete success publish current state and return its generation.

Any failed initialize/recover/set/revert, including no previous value or exhausted
counter, leaves `get`, `generation`, `set` and `revert` requiring recovery. Earlier
returned values are historical observations after an error. A full write followed
by an error may have persisted the new generation; the core confirms neither the
old nor new value until successful recovery. No failed attempt returns a committed
acknowledgement. Readback mismatch also retires state. Recovery after a torn write
can select the older valid generation; this is intentional integrity recovery,
not authentication or rollback resistance.

## Canonical media layout and trust boundary

A media adapter supplies exactly two 96-byte logical slots. Reads fill a slot
or return an error. A write may fail after any number of bytes, including after
the complete write. Successful flush promises durability of previous writes;
a real adapter must establish that promise separately. The caller owns media
identity, exclusive single-writer serialization, user scope, authorization and
revocation. Use the same medium throughout recovery and subsequent operations;
this core does not bind an untrusted media adapter to a user or capability.

Each slot has the following fixed little-endian encoding:

| Bytes | Meaning |
|---|---|
| 0–7 | Magic `NOSSET01` |
| 8–11 | Schema version 1 (`u32`) |
| 12 | Previous-value-present flag, bit 0 only |
| 13–15 | Zero reserved bytes |
| 16–23 | Nonzero generation (`u64`) |
| 24–31 | Current settings |
| 32–39 | Previous settings, or all zero when absent |
| 40–63 | Zero reserved bytes |
| 64–95 | SHA-256 of bytes 0–63 |

An eight-byte settings encoding is: theme byte (0/1/2), reduced-motion byte
(0/1 only), `u16` text scale, then four zero reserved bytes. Unknown flags,
invalid values, zero generation and nonzero reserved bytes are invalid even
with a matching checksum. Each operation uses bounded fixed buffers; no allocator,
parsing of arbitrary collections, network or real-disk action is involved.

SHA-256 detects ordinary corruption and incomplete writes; it is **not a MAC,
authentication, encryption or rollback protection**. An attacker can replace or
recompute records, including an older complete image. A malicious/incorrect
adapter can misreport durability or media identity. Per-user encrypted storage,
monotonic anchoring, native crash/power-loss evidence and UI integration remain
separate work. The dependency reuses the existing app journal's maintained pin:
`sha2 = 0.10.9`, default features disabled, `force-soft` enabled.

## Verification

Use Rust 1.90.0 with `sdk/settings-core/Cargo.toml`: `test --locked`,
`clippy --locked --all-targets -- -D warnings`, `fmt -- --check`, and
`build --locked --release --lib --target x86_64-unknown-none`.

Synthetic-media tests exercise typed limits, reload/revert, canonical records,
every torn-write prefix when overwriting an existing slot, full-write-but-error,
full/flush/read errors, readback mismatch, both-corrupt copies, unknown valid
versions, strict padding/value checks, fallback versus I/O failure, and generation
exhaustion. The media is host memory; tests establish no native service, per-user
encryption, real flush durability, physical power-loss resilience or human UI
acceptance. SDK membership is pending separately; no unchanged SDK matrix is run.
