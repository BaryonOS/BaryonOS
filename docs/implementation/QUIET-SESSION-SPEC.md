# Quiet futurism 2.0 — session and workspace

**Date:** 2 November 2026. **Source baseline:**
`1f0dd9c93959e3b1335656da1e548525a288133e`. **Status:** design before visual
integration, no new test/QEMU evidence. Score **41/100 remains 41**.
The [route research](../research/SESSION-ROUTES-2026-11-02.md) supports familiar
conventions. This specification supplements [PRODUCT-SPEC](PRODUCT-SPEC.md),
[the session contract](SESSION-AND-DESKTOP.md) and
[the native window session](NATIVE-WINDOW-SESSION.md). Existing storage,
privacy and capability boundaries remain authoritative; no new roadmap or gate.

## Product correction

According to the user brief, the negative reference shows bare surfaces,
little hierarchy, similar blue-green shades, technical status lines and an
uninformative dock. The problem is missing session/window structure; glass,
gradients, rounded corners or motion do not resolve that. This is the
supplied criticism, not a new inspection of a screenshot file that was not
provided.

The current source already has proportional antialiased system lettering,
original icons, two app processes, focus/dragging/recovery and a separate
sign-in preview. This functionality remains. New current screenshots
determine which shortcomings still exist; earlier screenshot criticism is
not code evidence.

The composition gains three layers: permanent system status; a workspace
with Materiaal and tool windows; temporary search/system layers with correct
focus restoration. Materiaal remains an object with revisions,
interchangeable tools and revocable access. Launching an app grants no general
data access. Permissions, history and location belong with the object,
not in an administrative console across the entire desktop.

## Visual rules and tokens

Values are **design targets**, not already measured contrast/renderer results.
Native shared colours currently come from `ui/graphics/src/theme.rs`; compact
geometry/app colours partly from `ui/prototype/tokens.css`. The builder
changes the actual sources coherently; no third palette in a renderer.

| Role | Light | Dark | Function |
|---|---|---|---|
| Canvas | `#F2F1EC` | `#151C24` | calm workspace background |
| Raised surface | `#FFFFFF` | `#202A35` | menu/search/dialogue |
| Active window | `#FFFFFF` | `#263442` | active title/controls |
| Inactive window | `#E6E8E7` | `#1D2732` | readable, lower emphasis |
| Primary text | `#20262D` | `#F2F4F5` | content/labels |
| Secondary text | `#5C6570` | `#B8C1CB` | metadata/location |
| Accent | `#14655E` | `#79C6BB` | restrained primary action |
| Selection | `#DCEDE8` | `#254840` | also shape/text |
| Focus | `#275DAD` | `#A8C8FF` | continuous border |
| Success | `#236A47` | `#8AD4A8` | also text/icon |
| Warning | `#88520A` | `#F0C475` | consequence/recovery |
| Danger | `#A13B32` | `#FFB1A6` | rejection/irreversible action |
| Offline | `#5C6570` | `#B8C1CB` | “Offline · available locally” |
| Trusted system layer | `#15232F` | `#101922` | compositor header/badge |
| Modal backdrop | `#15232F` at 32% | `#000000` at 44% | dim and block input |
| Desktop ink | `#18242F` | `#111A23` | dark outer shell |

Dark tokens define roles; a complete dark-mode port is ready only after
review of all states. Colour grants no authority: an app can draw dark
content; only compositor, input focus and capabilities protect trusted UI.

- Base unit 4 logical pixels; spacing 8/12/16/24/32/48.
- Controls at least 44 logical pixels high. Icons on a recognisable 24-unit
  grid, with a label if meaning is otherwise not obvious.
- Radius 8/12/16 for control/window/dialogue; no pill shape or card around
  everything.
- Window border 1 logical pixel; focus border 3. A subtle shadow supports
  stacking order but replaces neither a title nor an active indicator.
- Body target 16/24, metadata 14/20, section 20/28, title 32/40; lock time
  88/96 where space permits. Body weight 400, title 600. No general uppercase
  or monospace. Actual font metrics determine alignment, not character counts.
- Native reuses the pinned Open Sans atlas. New optical sizes require actual
  atlas/scaling tests; enlarged pixel letters are not high-DPI acceptance.
- Editable document text uses the same proportional glyph measurements for
  selection, wrapping, caret, pointer and navigation. UTF-8 bytes never change
  to fit the design. The bounded atlas/fallback does not yet support complete
  text shaping, IME or native screen-reader acceptance.
- Motion explains cause/location, with a target maximum of 120–160 ms.
  Reduced motion switches immediately; no animation/sleep artificially
  delays readiness or input.

## Board with eight coherent states

The board is a **design contract**: before final integration, create one
inspectable composition of the states below. A board render is neither a
runtime image nor native evidence. Then capture the same states with actual
interactions and assess their overall coherence.

| State | Composition | Interaction and evidence boundary |
|---|---|---|
| 1. Locked | clear time/date, device, limited power/network status; no private work | “Sign in”, Enter, visible focus; clock from adapter; fixed fixture time only when labelled in board/tests |
| 2. Select user | compact vertical user rows, avatar/initials, name, last used without work details | Tab/arrows/Enter; no app-card grid; administrator status only where needed |
| 3. Authenticate | selected name, method, credential/hardware instruction, busy/error/recovery | actual broker boundary, visible test-adapter label; after success first unlock, then the correct session |
| 4. Empty workspace | fixed status/time/session menu, named launcher and Materiaal start action | no fake files or decorative service claims; accurate offline/unavailable states |
| 5. Two windows | Materiaal with a populated list/revision/location; Calculator with an actual calculation/history | active/inactive title, focus, close/hide/restore; content survives compositor restart |
| 6. Search | bounded field; groups for apps/Materiaal/settings/actions; labels and shortcut | type/Enter/Escape; result grants no missing capability; empty/error states included |
| 7. Quick settings | anchored to the status area; device/network/power/scale, entry to settings | only actual support enabled; no fake battery percentage or apparent network connection |
| 8. Capability dialogue | system header, program/object/action/destination/duration/consequence/revoke | cancel by default for risk; background input blocked; compositor-hosted variant requires separate native proof |

Board: 1440×900. Runtime images: primary native **1280×720**, 1440×900,
1024×768 and 1920×1080, the last also at 200% where supported. A browser
size does not prove a guest resolution; unexecuted native sizes remain open.

Layout uses logical pixels: 1920×1080/200% is 960×540 logical. Authentication
panel at most 480 wide; search layer at most 720 with a 24 margin. Smaller
screens receive a more compact composition or scrollable content area,
without a primary action outside the viewport. Deliberate window overlap
remains recognisable; clipped controls/system menu are a failure. Output
loss keeps system controls within safe bounds; multiple outputs are for now
only a future contract.

## Session route and adapter boundary

Existing `SessionFlow` retains correlation and erases owned credential buffers;
`AuthBroker` separates authentication/activation. `UnavailableBroker` remains
the safe default while the real service is absent. `VmGuest` is a public
synthetic VM workspace, not correct-password success, an unlocked user key
or a secure compositor lock. A test adapter exists only in an explicit
test profile.

```text
Boot → Preparing → Locked → UserSelection → MethodSelection
→ Authenticating → Unlocking → Desktop

Desktop → Locking → Locked
Desktop → LogoutPending → UserSelection
Desktop → Switching → UserSelection
```

`Preparing` uses actual readiness/recovery, not fake services or fixed sleeps.
`Unlocking` follows a valid authentication result; desktop input begins only
after successful session activation. Sensitive app input remains blocked
on error.

Existing enums are extended selectively behind the same broker boundary.
The names below describe the **target contract**, not an existing wire ABI:

```text
AuthMethod = Password | LocalPin | Fingerprint | Fido2SecurityKey | RecoveryMethod
AuthResult = Authenticated(session_assertion) | Rejected(reason)
           | TemporarilyLocked(retry_policy) | Cancelled | HardwareUnavailable
           | HardwareRemoved | Timeout | ServiceFailure
```

`session_assertion` is a service-issued single-use proof or opaque handle,
bound to user, boot, service/session generation, purpose, fresh challenge
and expiry. Correlation fields/public Rust constructors do not prove an
issuer. At activation, the session service rechecks issuer, revocation,
freshness and scope. No custom cryptography. A real service needs persistent
verifiers/enrolment and monotonic rate limiting; a UI countdown is not a
rate limiter.

Changing method/user, cancellation, lock, logout and a relevant service error
retire the attempt. Late replies and reused assertions fail closed. Error
text reveals no credential details. Hardware text may show “Key removed”
without registered credential metadata. No passwords, private keys,
biometric templates or hashes of credential input in logs/screenshots/
fixtures. Tests record only synthetic public states.

## Desktop and access controls

Permanent layer: launcher/search, open apps, clock/status, notification centre,
quick settings and session menu. Lock, switch, sign out, shut down and restart
are named actions. An unavailable power service does not report “shut down”.
Open work follows save/discard/cancel and actual commit/flush according to
existing storage contracts.

The compositor owns system borders, titles, z-order, bounds, focus,
minimisation and input routing. Apps draw only their own surface content.
Moving/scaling preserves bounds/dirty regions. Alt+Tab switches windows;
controls are also keyboard accessible. Closing with a draft follows the
dirty check; Escape erases nothing. Restart retires old input/presentation
epochs; apps retain content and accept only newly authenticated bindings.

Search shows only authorised providers; no names from another user. Existing
grants are reused; installation grants no data permissions. A capability
dialogue names app, object/device, action, destination, duration, consequence
and revocation. For a risky read-plus-internet combination, focus starts on
cancel. Revocation changes broker/kernel policy, not just the UI.

| Mechanism / threat | Invariant, trusted component and enforcement | Error/recovery and open evidence |
|---|---|---|
| Authentication forgery/replay | authentication/session service validates user/boot/purpose/challenge/expiry; UI grants no permissions | no key release/desktop; attempt retired, reauthentication; real service/keys open |
| App imitates system dialogue | compositor owns trusted surface, non-overwritable system identity and exclusive modal input; kernel binds the actual peer | app claim/stale action rejected, valid focus return; browser modal does not prove the native boundary |
| Input to another session | input service/compositor check session/owner/epoch/bounds; apps have no InputSource/DisplaySource | drop/deny, fresh rebind after recovery; new lock boundary requires native tests |
| Rights expansion | capability broker/kernel bound object/action/destination/duration | rejection visible, draft preserved; new prompt only for new scope/authority; gaps remain named |

## Accessibility and visual acceptance

Tab/Shift+Tab follow reading order; Enter activates the intended control;
Escape closes only the top temporary layer or cancels an attempt. Focus
remains visible and within modality; return to the trigger or a safe successor.
Icon-only controls have labels; colour is not the sole carrier of status.

Browser: landmarks, actual buttons/fields, dialogue names and bounded live
announcements. Native builds on the existing semantic tree with node ID,
parent/role/name/bounds/focus/enabled/actions. That is a future broker hook,
not an OS-wide screen reader, IME, bidi or complete shaping. WCAG for browser
and native design is not a conformance claim: W3C, 12 December 2024,
[WCAG 2.2](https://www.w3.org/TR/WCAG22/).

Each runtime image receives source commit/evidence level/resolution/scale/
input route/artifact hash. Review before/after for hierarchy, alignment,
readable text, active windows, focus, contrast, clipping, object status,
information density and unwanted technical main text. Explicitly assess
recognisable time/status/search/windows; a passing pixel test or isolated
authentication page is insufficient. AI image review is not a human user trial.

## Execution and acceptance

| Group | Required trials | Evidence |
|---|---|---|
| Session | wrong password/user, success before activation, replay/stale/wrong boot/session, rate limit, cancel, key removed, timeout, service crash, no fail-open | negative host tests first; then actual services natively |
| Shell | complete route; search for four target types; two windows; status/session menu | browser/native separately, adapter labelled |
| Compositor | missing surface capability, trusted-UI imitation, background input during modality, wrong session/epoch, focus return, crash/rebind | actual QEMU tests; host model is preparatory |
| Accessibility | Tab/Shift+Tab/Enter/Escape, trap/return, 100/200%, reduced motion, labels/roles, draft intact | browser checks/native hooks separately |
| Visual | eight board states/runtime images, no clipping/miniature controls/technical main layer | explicit visual review alongside regression tests |
| Speed | boot→first frame, boot→lock/input-ready, authentication success→desktop/input-ready, full/partial render, dirty pixels/IPC; capture/QMP separately | before/after with the same configuration, actual run/log binding |

Designed, host-tested, browser prototype, native QEMU and physical hardware
remain separate. No figures without a run, no U02/FR-S02 points for a
board/adapter, no new claim for already accepted U01. The integrator maintains
shared status, score and architecture decisions; this specification is not
a second roadmap.

Open: **P0** coherent lock/login/desktop with actual adapters; **P1** compositor
lock/trusted modality/session input/sizing; **P2** everyday settings,
notifications/power/recovery with persistence; **P3** actual FIDO2/biometric
devices, multiple displays and physical hardware.
