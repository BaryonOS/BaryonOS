# Profiles and multiple users — step plan

Date: 30 October 2026. Status: **design; user model decided**
(ADR-058 in [DECISIONS.md](../DECISIONS.md)). It has been decided that setup,
sign-in and user management must be recognisable to consumers coming from
Windows, macOS and Linux. The detailed choices were left to the design;
§3 records them. No code has yet been written for this plan and nothing has been
tested natively. The desktop itself is described in [DESKTOP-SHELL.md](DESKTOP-SHELL.md).
Source base: main `c0b7147b`. Progress proposal: **34% → 34%**; this document
closes no criterion.

## 1. Motivation and desired outcome

The current native test build shows one app (Materiaal): create a document,
type, save and reopen. There is no first-start setup, no profile,
no sign-in or lock screen, no settings screen and no second
person on the same PC. main contains no code for onboarding, profiles or
user sessions. The corresponding release rows are open in
[FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md):

- **FR-S01** — first use and setting up a local profile (rubric U01, U05, B01);
- **FR-S02** — locking and unlocking the session with real authentication (U01, U05, B02);
- **FR-S03** — changing settings and finding them again after a restart (U05, S08).

Desired outcome for a consumer: at first start, create a personal profile in a few
steps, without a vendor account; afterwards sign in,
lock and switch between multiple people on the same PC, while
everyone can read only their own data.

There is as yet no design for **multiple people on one device**. The
P2 simulation (S01 `#/start` in [PRODUCT-SPEC.md](PRODUCT-SPEC.md)) has one
local profile without real password entry; [DESIGN-AMBITION.md](../DESIGN-AMBITION.md)
and [LOCAL-PRIVACY-CONTRACT.md](../LOCAL-PRIVACY-CONTRACT.md) describe
multiple *devices* of one user and NAS guests, not multiple local
users.

### Observations from the manual trial (agent, not acceptance)

On the Windows PC concerned, bundle `eabc27b`, Windows QEMU 11.1.0, TCG:

- The first image appeared after about 30 s. Enter on the start screen
  did not open a document after 15 s; a mouse click on **Nieuw** (New) did. A
  guest mouse pointer was not visible in the screenshots.
- The screen visibly refreshes strip by strip ("magnetic
  drawing board"). This matches the existing design: Materiaal performs one
  tile transaction per bounded iteration ([NATIVE-MATERIAAL.md](NATIVE-MATERIAAL.md))
  and the compositor blits each tile via `DisplayClient.blit` to the
  screen ([NATIVE-COMPOSITOR.md](NATIVE-COMPOSITOR.md)). The kernel writes
  those tiles directly (volatile) into the visible GOP framebuffer
  (`kernel/src/framebuffer.rs`); there is no back buffer, page flip or vsync.
  That explains the visible build-up; the refresh time has not been measured.
- Input is polled: IRQ1/12 does not yet wake the input wait
  ([NATIVE-INPUT-IDLE-WAIT.md](../contracts/NATIVE-INPUT-IDLE-WAIT.md)).
- A separate trial with `-accel whpx` started up to `MATERIAAL:FOCUS:READY` and
  drew the start screen; QEMU reported `Ignoring request for interrupt
  vector 0`. Input under WHPX has not been tested. WHPX is not an accepted
  profile.

These points are one user session and screenshots, not a measurement or
user research.

## 2. Existing building blocks

| Building block | What exists | What is missing for profiles |
|---|---|---|
| [B01 Key Service](B01-ENTROPY-AND-KEYS.md) | Entropy, master key, two fixed domains with sixteen key indices each (B01 trial boundary), revocation, recovery secret without escrow | No key domain or master key per user, no TPM protection, no password KDF |
| [B02 records](B02-SECURE-RECORDS.md) | Authenticated records; persistent key-state blob with one `master[32]` per volume under an external 32-byte secret; "a separate user/recovery agent supplies the external secret" | That agent does not exist; no key state per user; cold rollback anchoring and write-ahead are open |
| [Encrypted storage](ENCRYPTED-OBJECT-STORAGE.md) §5 | Target model DEK per object; Argon2id (RFC 9106) as preferred candidate for password recovery | No implementation or measured parameters |
| Volume/catalogue (M6) and document session | Native objects, catalogue and commit | No namespace per user; no encryption at rest (FNV-64 only detects corruption) |
| Capabilities, compositor, input router | Focus only via the trusted compositor; apps cannot inject input | Compositor serves one assigned surface; no session owner, secure input route, lock screen or user switching |
| [TPM CRB transport](../contracts/TPM2-CRB.md) | Bounded transport core, opt-in fixture | No product grant or key binding |
| [NAS management model](NAS-SHARING-AND-INTERNET.md) | Administrator may set quota and wipe, not read | Not yet applied to local users |

## 3. User model and fixed choices (ADR-058)

Guideline: what Windows, macOS and common Linux desktops have in common. Where
they differ, the chosen variant is stated below.

| Role | May | May not |
|---|---|---|
| Administrator (the first account) | Add/remove users, allocate space, wipe a user and their local data, approve system settings and updates | Read content, document names or keys of other users; reset another user's password while retaining their data |
| Standard user | Manage own files, settings, apps and permissions | Change other users or system administration without the administrator password |
| Guest | Temporary session; everything is wiped on sign-out | Keep anything or see other users |

| Topic | Choice | Familiar from |
|---|---|---|
| First account | Becomes administrator; may remain the only user | All three |
| Administrator actions | A standard user or app asks for the administrator password in a trusted system window (dimmed background) | UAC, macOS administrator prompt, `sudo`/polkit |
| Password | Mandatory at setup; no password hint | All three |
| PIN | Optional, at least 4 digits, with increasing delay after failures. Without TPM binding the PIN only unlocks a locked session and never protects keys on disk; after a reboot the password is required, with that explanation on screen | Windows Hello PIN; "password required after restart" on macOS/phones |
| Locking | Keys remain in memory; sign-out, shutdown and guest sign-out wipe them | All three |
| Automatic locking | By default after 5 minutes of inactivity, configurable | All three |
| Guest account | Available, off by default; can be turned on in Settings > Accounts. Guest data is held under an ephemeral key that exists only in memory | macOS, Linux |
| Recovery | Show the recovery key at setup and have it stored; later a second own device. No vendor account, backdoor or escrow | BitLocker/FileVault recovery key |
| Vendor account | Never mandatory; not silently via the network either | Local account on all three |
| Sessions | First version one active session; switching = lock and choose another user. The locked session continues to exist and falls under invariant 9. Visibly concurrent work follows later | Fast user switching |

### First start (OOBE: out-of-box experience)

1. Language and region.
2. Keyboard layout, with a field to test it.
3. Network (Wi-Fi/cable) with a clear **Skip for now** button.
4. Create account: full name, account name (suggested from the name),
   password and confirmation, optional profile picture.
5. Show recovery key; confirm it has been stored (print, save to USB or
   copy by hand); continue only after confirmation.
6. Optionally set a PIN.
7. Storage: by default only on this device; pairing an own NAS can be done now
   or later (FR-S01). There is no vendor storage as a default.
8. Privacy: one screen explaining that Nieuw OS sends no telemetry;
   location and similar permissions are off by default.
9. Done: the desktop appears with a short, skippable tour.

Going back is possible at every step. An interruption leaves no half account behind;
after a restart, setup begins at step 1 or at the last completed step,
without half-created keys.

### Signing in, locking and signing out

- **Lock screen:** background, clock and date; no notification content.
  A key press, click or swipe shows the sign-in screen.
- **Sign-in screen:** drawn by the session service, not by the shell or an
  app. The most recently used user is preselected with a password
  or PIN field; other users (and guest, if enabled) are shown as a list
  with name and picture. **Forgot password?** leads to recovery with the
  recovery key. Power button with shut down and restart.
- **Power menu in the start menu:** lock, sign out, switch
  user, sleep, restart, shut down.
- **Keyboard shortcuts:** OS key+L locks; Ctrl+Alt+Delete opens a
  trusted system screen with lock, switch user, sign out and
  task manager. Ctrl+Alt+Delete is the secure attention sequence: the
  input router delivers that combination exclusively to the session service, never
  to an app or the shell. An administrator password for an action by
  a standard user is also entered on a surface of the session service
  (invariant 8).
- **Settings > Accounts:** change password and PIN,
  profile picture, automatic locking, manage users and guest
  (administrator).

## 4. Threat model and invariants

Protected against (goal; only demonstrated after steps 2–5): another local
user, including an administrator or someone with their own password and the disk;
a malicious or compromised app, including one that imitates a sign-in
window; and reading a powered-off PC or a detached disk. Today
the document volume is not encrypted; that protection therefore does not yet exist.
Not protected: a compromised kernel or trusted system service,
physical attacks on a running PC, and, without TPM binding, an offline
brute-force attack on the password with the disk in hand (Argon2id makes that
more expensive, not impossible). These boundaries must be visibly stated in the explanation.

Invariants (design first, then interface, then code and negative tests):

1. An app receives capabilities exclusively within the namespace of the
   signed-in user; a guessed object ID, catalogue name or handle of
   another user yields nothing.
2. The password does not leave the session service, is not logged and is not
   directly a key. Via Argon2id it unlocks a random
   secret of **that user**, which only unlocks that user's own key state (own
   master key and DEKs). There is no shared volume key
   with which one user's password decrypts another user's
   content. The PIN never protects keys on disk.
3. A wrong password, aborted attempt or I/O error unlocks nothing and
   grants no partial access (fail closed); attempts are rate-limited.
4. While locked, apps receive no input and no app can draw over the
   lock screen; only the trusted session service owns that surface.
5. The owner can wipe a user without that user's secret, but has no
   operation that decrypts user content.
6. An interrupted setup leaves no half profile behind.
7. A user's content and document names are stored encrypted on disk with
   that user's own keys ([ENCRYPTED-OBJECT-STORAGE.md](ENCRYPTED-OBJECT-STORAGE.md)).
   Anyone who has the disk and their own password cannot decrypt another user's objects and
   names.
8. A password, PIN or administrator password is entered exclusively
   on a surface of the session service, reached via the
   lock screen or the secure attention sequence (Ctrl+Alt+Delete). The shell
   and apps never receive those keystrokes. An app can imitate pixels;
   therefore the system never asks for a secret outside that route.
9. On user switching, apps of the locked user receive no
   input, do not draw on the active user's screen and do not read
   that user's surfaces, clipboard or objects. Conversely, the new
   user cannot use the key state, objects and capabilities of the
   locked user either; see the transition below.
10. Guest data is held under an ephemeral key that exists only in memory.
    After sign-out, a crash or power loss it is unreadable and the
    next start cleans it up.

### Key transition A to B and back

This is a contract to be implemented, not an existing session service:

- Before B becomes active, the session service locks A and renders A's
  session-bound input, surface, clipboard and key-operation handles
  unusable via revocation or generation checking. An error along the way keeps
  the transition locked; no partial B session is activated.
- A's retained key material remains exclusively in A's isolated
  key domain. Every key operation checks the immutable
  user identity and current session generation. B receives no
  key handle, memory mapping or object grant of A; there is no global
  'current user' key context that redirects old handles to B.
- Pre-authorised background synchronisation may only continue with its own
  delegation, separately bound to A. A user switch does not
  grant or broaden that delegation and never makes it usable by B.
- Returning to A requires authentication again. Only after that are
  new session handles issued within A's existing permissions; old
  handles do not silently become valid again.
- If this separation cannot be guaranteed, switching is refused
  until A explicitly signs out; signing out wipes A's keys. The interface reports
  that boundary, without silently discarding unsaved work.

Native negative acceptance for A -> B -> A: B cannot use A's key state, objects
and old handles, A's apps receive nothing from B, stale handles
remain invalid after returning, and an error between revocation and activation
leaves both domains locked. Check permitted A background sync
separately for the same identity, scope and revocation, without B access.

No home-made cryptography: reuse the existing
XChaCha20-Poly1305/HKDF-SHA-256 from the Key Service. An Argon2id implementation
is only added after the dependency check (provenance, licence,
maintenance, `no_std`, build scripts, pinned version); no crate has been chosen yet.

## 5. Steps

Each step is only done with native QEMU evidence, including the failure cases.
Host tests, a build or an AI review are not acceptance.

| Step | Result | Depends on | Acceptance (native, with negative cases) |
|---|---|---|---|
| 0. Usable controls | Visible mouse pointer; keyboard activation on the start screen; show a frame only when it is complete; redraw only changed regions; input via IRQ1/12 instead of polling only | Existing input and Materiaal contracts; a display/compositor contract change for a back buffer or complete-frame commit (the kernel currently writes directly into the visible framebuffer) | Measured time from key press to visible pixel in the fixed TCG profile, before and after; no half-drawn frame visible; existing pixel, focus and input gates stay green |
| 1. User model decision | ADR-058 and §3–4 of this document | — | **Decided** 30 October 2026; security review of the invariants follows at step 3 |
| 2. Foundation | B02 cold rollback anchoring and write-ahead; key state and master key per user (more than the current two B01 domains); per-user encryption at rest of objects and document names; Argon2id with measured, versionable parameters; persistent settings storage | B01, B02, volume, ENCRYPTED-OBJECT-STORAGE | Restart preserves key state and settings; old snapshot, wrong password and corrupted record are rejected; offline trial: with the disk image and A's password, B's objects and names cannot be decrypted (independent oracle) |
| 3. Session service | Ring-3 service that registers users in encrypted form, verifies the password and issues capabilities per user | Step 2 | User A cannot open B's catalogue/objects, not even with guessed IDs; rate-limited attempts; no secret in logs or serial; an app or the shell receives no keystrokes from a sign-in or administrator prompt; an app that imitates a sign-in window obtains no secret; an administrator action by a standard user is refused without the administrator password |
| 4. First start (FR-S01) | Native setup: language/controls, name, password (mandatory), optional PIN, recovery key, storage choice | Steps 0 and 3 | Back/cancel; failure cases (empty or mismatching passwords, setup without password refused); power interruption during setup without a half profile; then start locally |
| 5. Sign in, lock, switch (FR-S02) | Sign-in screen, locking, switching user | Steps 3 and 4 | Repeated lock/unlock; wrong secret; aborted attempt; input race; increasing delay; PIN refused after cold restart; locked apps receive no input and do not draw over the lock screen; when switching, A's apps draw and read nothing of B (invariant 9); recovery with recovery key: wrong or revoked key, another user's key and interrupted reset give no partial access |
| 6. Settings and administration (FR-S03) | Per-user settings; owner adds and removes users; guest account on/off | Steps 2–5 | Setting survives cold restart; change and revert; invalid value, full storage and I/O error without false confirmation; wiping by owner without read access; guest data unreadable and cleaned up after sign-out and after crash/power loss |
| Later | TPM binding on supported hardware, second own device, NAS sync, concurrent sessions, possibly parental controls | Hardware profile, NAS service | Separate contracts |

Step 0 can run in parallel with step 2: it touches other files and resolves the
usability complaints that would otherwise make a sign-in screen unusable.
Steps 4 and 5 draw in the house style and screen layout of
[DESKTOP-SHELL.md](DESKTOP-SHELL.md); the desktop appears only after sign-in.

## 6. Evidence boundary

This document is a plan. It proves no working profile, no secure
authentication and no usability. User research (U06), an
independent human security review and physical hardware remain
separate requirements. On completion of each step, PROJECT-STATE,
the progress meter and, where necessary, DECISIONS are updated in accordance with AGENTS.md.
