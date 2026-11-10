# User selection, sign-in and desktop

**Date:** 2 November 2026. **Status:** researched interaction contract and
bounded `no_std` implementation; this change does not yet implement real account
authentication or session/key permissions. It does not close FR-S01, FR-S02 or
FR-U01–U05 and does not change the fixed progress score.

## Research: familiar controls

The official Windows guide describes selecting an account, using a password/PIN
or a configured Windows Hello method, confirming and opening the desktop.
Locking preserves applications; signing out ends the session; switching can
open another session without ending the first. Source: Microsoft,
*User Account Access in Windows*, accessed 2 November 2026,
https://support.microsoft.com/en-us/accounts-billing/security/user-account-access-in-windows.

On macOS, the user selects an account and authenticates. Touch ID can unlock or
switch a session previously opened with a password; a password remains required
at startup. Apple also specifies periodic reauthentication and a password after
five failed fingerprint attempts. Source: Apple,
*Use Touch ID on Mac*, accessed 2 November 2026,
https://support.apple.com/en-nz/guide/mac-help/mchl16fbf90a/mac.

GNOME displays the password form after a name is selected. A supported scanner
with prior enrolment can use a fingerprint; the password remains available.
The lock screen shows time/status and provides access to authentication.
Sources: GNOME, *Log in with a fingerprint*,
https://help.gnome.org/gnome-help/session-fingerprint.html and *The lock screen*,
https://help.gnome.org/gnome-help/shell-lockscreen.html,
accessed 2 November 2026. This concerns GNOME, not every Linux desktop.

After sign-in, the same tasks are available in different forms:

| System | Launching, searching, switching and system status |
|---|---|
| Windows | Taskbar with Start, search, open/pinned apps and status |
| macOS | Desktop, menu bar, Dock and Spotlight |
| GNOME | Top bar, Activities with search, dash with favourite/open apps |

Sources, accessed 2 November 2026: Microsoft, *Customize the Taskbar in
Windows*, https://support.microsoft.com/en-us/windows/experience/personalization/customize-the-taskbar-in-windows;
Apple, *Get to know your desktop, menu bar, and Dock*,
https://support.apple.com/en-euro/guide/mac-help/mchlws12345m2/27/mac/27;
GNOME, *Visual overview of GNOME*,
https://help.gnome.org/gnome-help/shell-introduction.html.

**Project decision:** use familiar names, account cards, a clear password field
and sign-in button, together with a taskbar and search. The project's Materiaal
objects, tools and restricted permissions remain the underlying model. A new
colour or launcher does not replace the other product requirements. No source
code from these systems has been incorporated.

## Authentication has a separate service boundary

Disk/key unlocking and session access are separate decisions. FileVault can
require credentials during boot; BitLocker can request a PIN or startup key
before Windows. The UI may combine the controls without equating key release
with account authorisation. Sources:
Apple, *Volume encryption with FileVault in macOS*,
https://support.apple.com/en-au/guide/security/sec4c6dc1b6e/web;
Microsoft, *BitLocker countermeasures*,
https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/countermeasures,
accessed 2 November 2026.

The intended standard designation is **FIDO2**. USB presence or a touch alone
is not general account verification. CTAP distinguishes presence (UP) from
user verification (UV, for example a PIN or fingerprint). Microsoft's
FIDO2 device sign-in documentation addresses Entra joined and hybrid joined
devices with explicit configuration; it is not evidence for arbitrary local
Windows accounts. Sources: FIDO Alliance, *CTAP 2.3 Proposed Standard*,
26 February 2026,
https://fidoalliance.org/specs/fido-v2.3-ps-20260226/fido-client-to-authenticator-protocol-v2.3-ps-20260226.html;
Microsoft, *Enable FIDO2 security key sign-in to Windows*,
https://learn.microsoft.com/en-us/entra/identity/authentication/howto-authentication-passwordless-security-key-windows,
accessed 2 November 2026.

A future FIDO2 service must validate, among other things, fresh challenges,
credential-to-account binding, the signature, RP binding and required UP/UV.
Source: W3C, *Web Authentication Level 3 Recommendation*, 25 August 2026,
https://www.w3.org/TR/2026/REC-webauthn-3-20260825/.
This change implements no CTAP, biometric driver, password KDF, persistent
account database or key recovery. The default broker rejects all these methods;
there is no shared demo password and no arbitrary password acceptance.

## Bounded implementation contract

Target platform: the existing standalone x86_64/UEFI-QEMU baseline; Rust `no_std`,
with no allocator, new dependencies, hardware access or network in this module.
Write scope: `ui/native/src/session.rs`, `ui/native/tests/session.rs` and this
document. The integrator owns the native visual integration.

The main route is `AccountSelection → CredentialEntry → Authenticating →
SessionStarting → Desktop`. `Locked` hides every active identity;
`unlock()` displays credentials again and does not grant access. Generation
and request counters never wrap: exhaustion ends in `Unavailable`.

| Interface | Responsibility |
|---|---|
| `SessionFlow` | UI state, account/method, bounded input and current correlation |
| `AuthBroker::begin` | Trusted adapter sends the request to the authentic service; the password is only borrowed |
| `BrokerReply` | Reply from that adapter, bound to request ID, UI generation, account ID and method |
| `BrokerTicket` | Opaque service handle; no `Debug`, `Clone` or `Copy` |
| `AuthBroker::activate` | Session service revalidates the ticket and starts only the permitted session |
| `UnavailableBroker` | Safe default while the real services are absent |

A granted ticket does not yet open a desktop. Only successful session activation
sets the user identity to `Authenticated`. An error, timeout or denial leaves
no active identity. Changing account/method, cancelling, locking and signing
out clear password input and invalidate pending replies. An old reply must not
fulfil the new request.

Passwords use at most 128 bytes; overflow returns an error. The UI can query
only the length. Input and ticket bytes are erased at the end of their lifetime
using the existing pinned `zeroize 1.9.1` implementation. The native UI uses
the same no_std configuration as the SDK, without an additional crate or build
script. This protects erasure of the explicit owned buffers from being
optimised away; temporary copies, registers, crash memory and data outside
the owned buffers are excluded. The future secret service still requires a
controlled complete lifecycle.

The default `SessionFlow::new()` uses local counters, not a random session
identity. The real broker must therefore also bind the authentic IPC peer,
service instance and session generation; correlation cannot serve as authority
across different UI processes or a restart. Public Rust constructors check
only shape and correlation, **not the issuer**. `BrokerReply::granted` may be
called only after real service verification by the trusted adapter. A
compromised UI can modify its own memory; kernel capabilities and key release
must never rely on the UI state field.

`VmGuest` is an explicitly selected disposable VM profile without a password
or user keys. `begin_guest` and `activate_guest` form a separate route and
produce only `SessionIdentity::VmGuest`; this is not successful user
authentication. The native adapter may offer this route only in the visible VM
preview profile and may use only its associated synthetic workspace. This
module does not create or mount storage.

Actual lock/revoke and input/surface isolation remain responsibilities of the
session/compositor service. `SessionFlow::lock()` cannot itself revoke kernel
grants. Search results remain within the permitted local catalogue; searching
does not grant additional file or network permissions.

Additional learning source: OSDev Wiki, *Security*, W002262, local snapshot
according to the repository skill. The passage on system boundaries and
untrusted lengths provides orientation; primary specifications and project
contracts take precedence. No tutorial code has been incorporated.

## Acceptance and evidence boundary

Targeted host tests check: explicit guest identity, rejection of all passwords
by the default broker, no desktop before activation, denial and timeout, failed
activation, cancellation/switching/lock with delayed replies, complete
account/method binding, retry with a new request ID, input bounds and secret
length cleanup. A service fixture in the tests is not a real authentication
service.

Command for the targeted suite from the repository:

```sh
source /workspace/.nieuw-os-env/activate.sh
CARGO_TARGET_DIR=/workspace/nieuw-os/build/session-flow cargo test --manifest-path ui/native/Cargo.toml --test session --offline
```

Executed on the worktree above source `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`:
the targeted suite passes with **10 tests, exit 0**; log
`/workspace/.nieuw-os-env/session-tests.log`. `rustfmt --edition 2021 --check`
for the session module and tests, and `git diff --check`, also pass, exit 0.
The first strict `cargo clippy --test session --offline -- -D warnings` was
blocked by concurrently modified integration code: unused `label` in
`render.rs` and `unnecessary_unwrap` in `desktop.rs` (exit 101; log
`/workspace/.nieuw-os-env/session-clippy.log`). This is not a passing combined
lint result; the integrator repeats the gate after completing those files.
Native UI acceptance, real account/key persistence, FIDO2/biometrics, kernel
revoke, physical hardware and human assessment are outside this host evidence
and remain open.
Rollback: remove the new session module/tests and the integrator reference;
this document retains the research contract. No existing user data is migrated
or changed.

## Native integration by the integrator

`native-system-session-preview` is an explicit opt-in VM profile. The kernel
sets only startup word 648=1; no new capability or cryptography. The native app
activates the shell over its existing synthetic app domain. The first five-step
trial on 7648a8f passed: all pixels in the app area (16,48,768,536) matched the
bound host renderer. The expanded final trial also checks personal test input,
rejection and erasure before explicit guest sign-in. Evidence and definitive
source are under
[session-desktop-20261102](../verification/session-desktop-20261102/).

The source-bound host oracle is a comparison, not independent native evidence.
Only real QMP input and QEMU screendumps demonstrate execution. The visible
account card 'Eigen gebruiker' (English: 'Own user') is not yet a registered
profile; a real authentication/session service must later add enrolment, a
persistent verifier, recovery, rate limiting, capability revocation and
compositor/input locking.

## Start surface — 3 November 2026

The VM preview now uses a hierarchical start surface with a recognisable
workspace, explicit navigation and one primary entry point to Materiaal. The
Materiaal card reads the existing list state of the real app and displays the
not-loaded, empty, error or document state; it creates no example files, recent
activity status or invented network/power status. The compact layout keeps the
card above the navigation bar and replaces a textless Start button with a
named action.

This is a visual and information change within the existing explicit VM guest
preview. It adds no account authentication, storage permissions, search
results, network status or new session transition. The empty and populated
card states have been added as native-renderer host regressions; local
execution requires a C linker. A host compilation/lint check is not a pixel
trial; the native QEMU/screenshot gate must pass on the exact commit before
the new display counts as evidence of native execution. The score remains
41/100 until a complete rubric criterion is accepted with appropriate evidence.
