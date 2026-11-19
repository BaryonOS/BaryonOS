# Session routes: reference patterns and concrete Nieuw OS choices

**Historical English equivalent.** Research date: **2 November 2026**. Source baseline: `1f0dd9c93959e3b1335656da1e548525a288133e`. Status: source research and design before new visual integration; no new execution acceptance. Historical score: **41/100 remains 41**. Accepted criterion U01 is distinct from full release requirement FR-U01 and secure-session requirement FR-S02. The [unchanged Dutch original](SESSION-ROUTES-2026-11-02.md) has SHA-256 `43e6daad2389a05011d8af924c2a403188c1500fd4f738f351cf83bd9769ac58`.

The examined code already had its own native window session, two isolated applications, focus/dragging, proportional system fonts, recovery and a bounded clipboard. `ui/native/src/session.rs` contained `SessionFlow`, `AuthBroker`, `UnavailableBroker` and a separate public `VmGuest` identity. The default broker refused actual authentication; a screen or account card granted no kernel authority. See the [session contract](../implementation/SESSION-AND-DESKTOP.md) and [revised visual specification](../implementation/QUIET-SESSION-SPEC.md).

## Method and boundary

Primary user guides supported the reference routes; technical documentation supported service boundaries. Availability depended on version, hardware, enrolment and policy. Omission of a method from this study did not establish its absence. No new comparative user study, security audit or hardware test was performed. Direct Apple guide requests returned HTTP 429; those guide observations were limited to retrieved official search-index excerpts. Reference images were not OS installations tested in this research.

“Researched pattern” describes the source observations. The following project choices and acceptance requirements were design work, with no claim that the corresponding implementation was complete. All reference observations below retain their 2 November research date. Current normative decisions and implementation status remain in the [accepted decisions](../DECISIONS.md) and [project checkpoint](../PROJECT-STATE.md).

## 1. Windows 11: identity, available method, desktop

**Researched pattern:** select an account, use a password/PIN or configured Hello method, confirm, reach the desktop; available methods vary by configuration. The taskbar combines applications/search/open work/status; Snap supports window arrangement. Sources: Microsoft, n.d., Windows 11/10, [User Account Access](https://support.microsoft.com/en-us/accounts-billing/security/user-account-access-in-windows), [Sign-In Options](https://support.microsoft.com/en-us/accounts-billing/security/sign-in-options-in-windows), [Taskbar](https://support.microsoft.com/en-gb/windows/experience/personalization/customize-the-taskbar-in-windows), [Snap](https://support.microsoft.com/en-us/windows/experience/snap-your-windows).

- **User problem:** uncertainty about the sign-in identity, available sign-in method and location of previously opened work.
- **Nieuw OS adopts:** identity name before credentials, named methods, a fixed status/search entrypoint and clear active windows.
- **Deliberate difference:** start locally without a vendor account; keep web results separate from personal Materiaal results; retain an independent identity.
- **System contract:** authenticated broker reply, local typed search results and compositor ownership of focus/bounds.
- **Location:** `ui/native/src/{session,desktop}.rs`, `sdk/examples/window-session/src/scene.rs`, `ui/prototype/`.
- **Acceptance:** a wrong account/method opens nothing; keyboard search finds applications, Materiaal, settings and session actions.

## 2. macOS: system functions have a stable location

**Researched pattern:** a menu bar with status, Spotlight, Control Center, privacy indicators and notifications; a system menu for settings/lock/power. User switching is available through the menu bar/Control Center; accessibility is reachable before login; startup/recovery states are recognisable. Sources: Apple, n.d., the consulted Mac guide, [Menu bar](https://support.apple.com/en-gb/guide/mac-help/mchl4af84660/mac), [User switching](https://support.apple.com/en-ie/guide/mac-help/mchlp2439/27/mac/27), [Accessible login](https://support.apple.com/guide/mac-help/mchlaa57f797/mac), [Startup recovery](https://support.apple.com/en-ca/102675).

- **User problem:** system functions must remain available across application/user changes; assistance must be reachable before authentication.
- **Nieuw OS adopts:** a stable system identity, time/status, session menu, accessibility entrypoint and causal session feedback.
- **Deliberate difference:** no global application menu bar or glass effect as the defining identity; Materiaal keeps tools, history and access together.
- **System contract:** service-derived status and privacy-safe pre-login data; user switching retires old input; recovery does not open an account.
- **Location:** system chrome in `sdk/examples/window-session/src/scene.rs`, `ui/native/src/session.rs` and browser routes.
- **Acceptance:** keyboard lock/switch; the old session receives no input; an unavailable service produces no success-shaped response.

## 3. GNOME: a quiet lock scene and explicit login

**Researched pattern:** a lock scene with date/time, selected notifications, battery/network; pointer, Enter or Escape opens login; switching is offered for multiple accounts. The top bar supplies status; Activities/Super/type finds applications and work; the dash distinguishes running applications. Sources: GNOME, n.d., [Lock screen](https://help.gnome.org/gnome-help/shell-lockscreen.html), [Visual overview](https://help.gnome.org/gnome-help/shell-introduction.html).

- **User problem:** see readiness without exposing private work; find applications and work from any screen.
- **Nieuw OS adopts:** a quiet lock scene with “Aanmelden” (Sign in), visible focus, a fixed search entrypoint and an open-work indicator.
- **Deliberate difference:** no mandatory application grid or hidden hot corner; no document titles/notification content before authentication by default.
- **System contract:** locked/users/auth/unlocking/desktop states and a privacy filter.
- **Location:** `ui/native/src/`; actual compositor lock later in `sdk/examples/window-session/`, outside the document application.
- **Acceptance:** Enter works without a pointer; Escape cancels an attempt and invalidates late replies; a document draft is preserved.

## 4. KDE Plasma: open work and search are ordinary desktop functions

**Researched pattern:** a panel with launcher, window list, system tray and clock; the launcher exposes applications/recent work/session actions; KRunner uses Alt+Space/Alt+F2. Sources: KDE, n.d., stable KF6 documentation, [Panel](https://docs.kde.org/stable_kf6/en/plasma-desktop/plasma-desktop/panel.html), [KRunner](https://docs.kde.org/stable_kf6/en/plasma-desktop/plasma-desktop/krunner.html).

- **User problem:** distinguish launching, already-open work and activation without learning an abstract icon vocabulary.
- **Nieuw OS adopts:** a named launcher/search function, active-application indication, a window switcher, recognisable title bar and recoverable minimisation.
- **Deliberate difference:** a populated Materiaal workspace with location/revision, rather than a widget collection or administration panels as the home screen.
- **System contract:** owner/visibility/z-order/focus/bounds per window; search results typed as App/Material/Setting/Action.
- **Location:** `sdk/examples/window-session/src/scene.rs`, application controllers, `ui/surface-protocol/` and browser interactions.
- **Acceptance:** launch two applications, switch, hide/restore/close; find “vergrendelen” (lock) without additional document rights.

## 5. SDDM: the greeter presents and the backend authenticates

**Researched pattern:** a separate greeter system user; screens/users/sessions exposed as models; authentication and power operations through callbacks. The theme is not the verifier. The screens model identifies outputs explicitly. Sources: SDDM, develop branch, [Theming](https://github.com/sddm/sddm/blob/develop/docs/THEMING.md), [README](https://github.com/sddm/sddm).

- **User problem:** visual customisation must not invent identity or success; login must fit the available display.
- **Nieuw OS adopts:** public user enumeration, a selected user, request correlation, busy/cancel/error states and explicit service outcomes.
- **Deliberate difference:** one contract for native/browser adapters; visibly labelled test adapters; no arbitrary demo password or silent guest fallback.
- **System contract:** account/method/request/boot/service generation, timeout, cancellation and separate activation; output-id/scale remains a future contract.
- **Location:** `ui/native/src/session.rs`, compositor/input adapter under `sdk/examples/window-session/`; a synthetic adapter only in a test profile.
- **Acceptance:** wrong user, cancellation, late reply, removed key, sensor failure or service crash cannot unlock an actual user session.

## 6. Android: authentication supplies evidence to another service

**Researched pattern:** Gatekeeper verifies within a TEE, throttles failed attempts and supplies a per-boot authenticated token to Keystore; biometrics use a separate provider. Quick Settings is central; multi-user/guest depends on support. Sources: AOSP, continuously maintained documentation, [Gatekeeper](https://source.android.com/docs/security/features/authentication/gatekeeper); Google, n.d., [Quick Settings](https://support.google.com/android/answer/9083864?hl=en), [Users](https://support.google.com/android/answer/2865483?hl=en).

- **User problem:** a UI boolean must not open an account or storage secret; hardware and services may fail during authentication.
- **Nieuw OS adopts:** service-side retry policy, bounded assertions and visible hardware/timeout/recovery states.
- **Deliberate difference:** no desktop swipe/pattern or custom matcher; a host model does not establish a TEE or working biometrics.
- **System contract:** account/boot/purpose/challenge/expiry/revocation; successful authentication alone does not establish successful key/session activation.
- **Location:** `AuthBroker` in `ui/native/src/session.rs`; future authentication/keyring services are enforcement points.
- **Acceptance:** replay/stale/wrong-session/rate-limit/empty mandatory PIN/crash cases fail closed; changing methods does not reset the limiter.

## 7. ChromeOS: one stable search and system entrypoint

**Researched pattern:** Search/Launcher supplies search/applications; the clock opens controls including lock, also available through Launcher+L. PIN/password and reauthentication depend on configuration/policy; the consulted help separately described a Chromebook password. Guest is a distinct route. Sources: Google, n.d., [Lock/unlock](https://support.google.com/chromebook/answer/2587994?hl=en), [Keyboard](https://support.google.com/chromebook/answer/1047364?hl=en), [Sign in](https://support.google.com/chromebook/answer/15641761?hl=en).

- **User problem:** find daily actions without remembering their application or settings page.
- **Nieuw OS adopts:** a stable search/action layer, status menu and lock action.
- **Deliberate difference:** start locally without a mandatory cloud account; being offline is a status, not failure of every local task. The research did not claim that ChromeOS always required a network.
- **System contract:** a local authorised index; session actions go to a broker; guest is offered only within an actually available isolated profile.
- **Location:** `ui/native/src/desktop.rs`, `ui/prototype/` and system chrome.
- **Acceptance:** offline search/open/lock; an unimplemented guest route does not produce a successful account sign-in.

## Local FIDO2: distinguish the standards

**Researched pattern:** Yubico `pam-u2f` supports local PAM login with public credential/account mapping, user presence, PIN and UV. Mapping must be available before unlock; placing it in a still-encrypted home can block login. `alwaysok`/`nouserok` are documented reference options, not Nieuw OS defaults. Source: Yubico, continuously maintained [pam_u2f(8)](https://developers.yubico.com/pam-u2f/Manuals/pam_u2f.8.html).

systemd-homed uses a different local unlock route: FIDO2 `hmac-secret` for an account/home with separate PIN/UP/UV policy, rather than a drop-in signature verifier. Source: systemd; the consulted manual reported version 262, [homectl](https://www.freedesktop.org/software/systemd/man/latest/homectl.html).

Microsoft's OS FIDO2 documentation concerns Entra joined/hybrid joined systems; it does not establish arbitrary local-account support. Source: Microsoft Learn, [Windows FIDO2](https://learn.microsoft.com/en-us/entra/identity/authentication/howto-authentication-passwordless-security-key-windows). Yubico describes key material on the authenticator, a challenge-bound assertion and public-key verification, without a claim about every synchronised passkey. Source: Yubico, [FIDO2 Credentials](https://docs.yubico.com/yesdk/users-manual/application-fido2/fido2-credentials.html).

- **User problem:** presence/touch is not a valid account verification; fallback can bypass policy.
- **Nieuw OS adopts:** enrolment, public verifier data, a fresh challenge, required UP/UV/PIN, revocation, cancellation and recovery.
- **Deliberate difference:** a test adapter at that stage; later a reviewed CTAP2/FIDO2 library and device capability; no custom cryptography, actual passthrough or keys in Git.
- **System contract:** the private key stays on the hardware authenticator; the broker validates user/RP/challenge/policy and performs separate session activation.
- **Location:** `ui/native/src/session.rs` as the UI boundary; a future authentication service owns verifier/hardware access, never the renderer.
- **Acceptance:** absent/removed/wrong key, stale/replay, timeout/cancel and missing required PIN/UV produce no assertion. A historical NULL-PIN bypass motivates the fallback test: Yubico, 2021, [YSA-2021-03](https://www.yubico.com/support/security-advisories/ysa-2021-03/).

## Boot, displays and measurement

Windows distinguishes firmware/Boot Manager/OS Loader/kernel; Android guidance supports phase analysis; Plymouth separates boot splash and diagnostics. Sources: Microsoft Learn, [Startup troubleshooting](https://learn.microsoft.com/windows/client-management/advanced-troubleshooting-boot-problems); AOSP, [Optimize boot times](https://source.android.com/docs/core/perf/boot-times); freedesktop.org, [Plymouth](https://wiki.freedesktop.org/www/Software/Plymouth/).

Nieuw OS selected early feedback and recovery tied to actual readiness milestones, without a fictitious percentage. A splash frame did not establish usable input.

Firmware/loader/kernel/services/storage/display/first-present/input-ready were to be measured separately on the same QEMU configuration. QMP/capture/hashes/verification and additional VM starts remained separate from rendering work. TCG and WHPX/KVM/HVF were different measurement arms. “45 seconds” was an observation to investigate, not a demonstrated renderer duration; the research introduced no new measurements.

The multi-display contract called for output-id, scale, safe bounds, input focus and recovery after output loss. An initial slice could have one demonstrated output; a screenshot did not establish multiple displays, scanout or physical hardware. All sources were consulted on **2 November 2026**; continuously maintained pages without a fixed revision date were treated as n.d. The design choices established no OS-version parity, certification or comparative security claim.
