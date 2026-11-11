# Privacy browser — research direction and open choices

**Status:** research note, 28 October 2026. This document records the product
requirement, relevant open-source examples and acceptance questions. **No
browser engine has yet been chosen, forked or ported to Nieuw OS**. Chromium/Blink,
Firefox/Gecko and their derivatives are references; a choice only becomes an
architecture decision after a targeted comparison and trial.

## 1. Product goal

For everyday use, Nieuw OS needs a browser that can handle ordinary websites,
web apps, video calls, payments and other common web tasks, without turning
behavioural profiling or advertising telemetry into a revenue or
system model.

The desired default behaviour is:

- the browser vendor receives no browsing history, advertising profile or
  usage telemetry that is not strictly necessary for an explicitly chosen
  diagnostic action;
- known cross-site trackers and tracking storage are restricted by default;
- fingerprinting is reduced where practically possible, without making the user
  unique precisely through exotic deviations;
- sensitive browser permissions such as camera, microphone, location, clipboard and
  files remain tied to a concrete user action;
- a website that does not work properly because of privacy protection gets an
  understandable recovery route, without the browser automatically disabling all
  protection;
- security updates and necessary background connections can be
  inventoried and explained.

This aligns with the existing privacy contract: an internet permission does not give a browser
any general document or NAS authority. The browser remains an ordinary,
bounded app on top of the capability and network boundaries of Nieuw OS.

## 2. Two separate privacy problems

### 2.1 What the browser itself sends

A privacy browser must first be able to account for its own network behaviour.
Investigate at least:

- telemetry, experiments, crash reporting and unique installation identifiers;
- search suggestions and text that may be sent to a search provider before Enter is pressed;
- update, certificate, filter-list, time and connectivity traffic;
- account, synchronisation and recommendation services;
- any Safe Browsing/malware checks and which metadata is shared
  with them;
- extension and component updates.

Goal: a clean installation has a documented list of necessary
background connections. Optional diagnostics are disabled unless the
user deliberately activates them.

### 2.2 What websites and third parties can collect

In addition, the browser must provide protection against tracking by visited
websites and embedded third parties. Candidates for the default profile:

- blocking known cross-site trackers and advertising trackers;
- partitioning or blocking third-party storage;
- stripping tracking parameters in URLs where this does not break navigation;
- protection against known fingerprinting vectors;
- limiting speculative/preconnect requests when they carry a privacy
  cost;
- clear site-data/cookie controls;
- no silent access to camera, microphone, location or local
  documents.

None of these measures can guarantee anonymity. Over a direct connection, a visited website sees,
among other things, the source IP and can link logged-in
activity to an account. Fingerprinting can be reduced but
not made universally impossible.

## 3. Temporary exceptions as a product feature

The proposed user route is not “turn privacy off”, but **“this website
does not work”**. The browser then attempts the smallest possible exception.

### Proposed behaviour

1. Protection is on by default.
2. When a site breaks, the browser shows which category is probably involved,
   for example blocked third-party storage or a tracker/script.
3. The first recovery action relaxes only that category for the current site.
4. The default choice is **one-time** or **until this site/tab is closed**.
5. A permanent exception is a separate, explicit choice.
6. Active exceptions are visible from the address bar and can be reverted in a single
   action.
7. An exception for site A does not automatically apply to the same third party
   on site B.
8. The browser can report a broken site with technical diagnostics after
   the user has been able to review their content.

### Not relaxed via this route

A privacy exception must not automatically disable the following security
boundaries:

- TLS/certificate validation;
- process sandboxing and site/renderer isolation;
- memory and capability boundaries of Nieuw OS;
- protection against downloads or executable code;
- the separation between browser internet access and personal documents/NAS;
- the rule that a website may never on its own grant additional browser or OS
  permissions.

A permission can, from that moment on, allow data to flow to the website or
third party concerned. Re-enabling protection cannot retrieve data that has
already been sent.

## 4. Relevant open-source examples

| Project | Base | Useful as a reference | Point of attention |
|---|---|---|---|
| Chromium | Blink/Chromium | Very broad web compatibility, multi-process architecture and sandbox design | The sandbox relies on security mechanisms of the underlying OS; a Nieuw OS port therefore requires its own sandbox integration |
| ungoogled-chromium | Chromium | Removes Google web-service dependencies and many background requests | Additional privacy tweaks are largely not enabled automatically; not a complete product strategy in itself |
| Brave | Chromium | Built-in tracker blocking, cookie/storage protection, fingerprinting measures and site-specific Shields | Its own product choices/telemetry must be assessed separately; do not adopt blindly as a base |
| Vanadium | Chromium | Example of browser hardening that works closely with OS security | Tightly coupled to GrapheneOS/Android; not directly a Nieuw OS port |
| Firefox | Gecko | Enhanced Tracking Protection, Total Cookie Protection and site-specific exceptions | Standard Firefox has its own product/telemetry choices that must be reassessed for Nieuw OS |
| LibreWolf | Firefox | Telemetry off, uBlock Origin, stricter tracking and fingerprinting settings | Strict defaults can affect site compatibility and certain services |
| Mullvad Browser | Firefox/Tor Browser | Compile-time disabling of telemetry and strong anti-fingerprinting; publishes background connections | A uniform fingerprinting strategy can carry UX/compatibility costs |
| Tor Browser | Firefox ESR | Strong anti-fingerprinting and network identity isolation | Tor routing/anonymity is a different product goal from a general everyday Nieuw OS browser |

This table does not pick a winner. Projects can contribute ideas without Nieuw
OS adopting their branding, complete patch set or product model.

## 5. Why Chromium may be technically attractive

Chromium is a serious candidate when maximum website compatibility and
support for contemporary web apps weigh heavily. The browser can be adapted as an
open-source base; ungoogled-chromium shows that
Google web-service dependencies and background requests can be removed
from Chromium.

A Chromium choice, however, does not just mean “porting the UI”. Chromium itself
describes that its sandbox uses security mechanisms of the
underlying operating system. For Nieuw OS, therefore, at least processes,
IPC, shared memory, network, file access, GPU/rendering, fonts, input,
time/entropy and the sandbox must be connected to the native kernel and capability
boundaries.

The port must moreover be able to keep following upstream security updates quickly.
A very large permanent patch set increases the risk that patches are delayed or
become difficult to integrate.

## 6. Why Firefox/Gecko must also be investigated

Firefox is technically relevant because Mozilla already has tracking partitioning,
site-specific protection and web compatibility mechanisms. LibreWolf
and Mullvad Browser additionally demonstrate that telemetry and additional
privacy features can be tightened further.

A Gecko choice does not solve the port to the native kernel by itself either. Here too,
graphics, network, process, filesystem, sandbox and update interfaces
are needed. The decisive comparison criterion is which codebase can carry the desired
product guarantees maintainably; visual preference is not leading.

## 7. Research criteria before an engine decision

Before an ADR with an engine choice, produce at least a reproducible comparison
of Chromium- and Gecko-based candidates.

| ID | Question | Minimum test |
|---|---|---|
| BR-01 | Can all non-essential product telemetry be removed? | Network capture of clean start, idle, search, browsing and shutdown; every background connection explained |
| BR-02 | How quickly can upstream security fixes be adopted? | Measure patch delta and build process across multiple upstream security updates |
| BR-03 | How often does the privacy profile break websites? | Fixed test suite with login, payments, streaming, video conferencing, web apps and embeds |
| BR-04 | Do targeted exceptions work? | Site-specific temporary exception restores function without a global privacy reduction |
| BR-05 | Which fingerprinting information remains visible? | Repeatable test set with documented APIs and configurations; no absolute anonymity claim |
| BR-06 | Can the sandbox be enforced on Nieuw OS? | Threat model and capability mapping for renderer, network, GPU, downloads and file portals |
| BR-07 | Is the dependency and licence chain manageable? | Inventory of source, licences, third-party code, build tools and update ownership |
| BR-08 | Does the experience remain consumer-grade? | Task trial with default profile and recovery after site breakage without technical settings knowledge |

## 8. Open product choices

Do not decide yet without a trial:

- Chromium/Blink versus Firefox/Gecko as the first engine.
- Blocking versus partitioning of the various forms of third-party storage.
- Anti-fingerprinting via uniformity, randomisation or a combination.
- Which phishing/malware protection can keep functioning locally or in a
  privacy-preserving way.
- DRM/Encrypted Media Extensions: privacy, freedom and consumer compatibility
  must be explicitly weighed against each other.
- Extension ecosystem and which extension API can be supported safely.
- Search provider, DoH/DNS behaviour and optional synchronisation.
- Whether browser passwords belong in the browser itself or in a separate
  local credentials service of Nieuw OS.
- Which diagnostics a user can view locally and voluntarily share for “website does not work”.

## 9. Proposed development order

**BR0 — comparison and privacy contract.** Run BR-01 to BR-08 first on
host builds. This yields an engine decision and patch budget, not a native
browser claim.

**BR1 — maintainable browser variant on the development host.** Build the chosen
privacy profile, temporary site exceptions and automated
network/compatibility tests. Keep the patch set small and rebaseable.

**BR2 — Nieuw OS platform layer.** Port the minimally required platform interfaces
to the native network, graphics, input, storage and process services. This
depends on sufficiently mature native network and graphics infrastructure.

**BR3 — sandbox and capability integration.** Renderers, network service, GPU/media
processes, downloads and file portals receive only the necessary
Nieuw OS capabilities. Test negative access and revocation end-to-end.

**BR4 — everyday-use acceptance.** Test real websites and tasks,
security updates, accessibility, performance, site breakage and recovery on the
supported hardware. Only then can browser functionality contribute to A04 and
relevant privacy/network criteria of the fixed roadmap.

## 10. Evidence boundary and progress

This note is research and product documentation. It implements no
browser, network stack or sandbox and therefore closes no existing
acceptance criterion. With this change the fixed project score remains
**14% → 14%**.

## Sources

Brave Software, Inc. (n.d.). *Brave Shields — Blocking ads, trackers & more*.
https://brave.com/shields/ (accessed 28 October 2026).

Chromium Authors. (n.d.). *Sandbox*. Chromium source documentation.
https://chromium.googlesource.com/chromium/src/+/main/docs/design/sandbox.md
(accessed 28 October 2026).

GrapheneOS. (n.d.). *Features overview: Vanadium*. https://grapheneos.org/features
(accessed 28 October 2026).

LibreWolf. (n.d.). *Features*. https://librewolf.net/docs/features/
(accessed 28 October 2026).

Mozilla. (2026, September 24). *Enhanced Tracking Protection in Firefox for
desktop*. https://support.mozilla.org/en-US/kb/enhanced-tracking-protection-firefox-desktop

Mozilla. (2026, June 24). *Total Cookie Protection and website breakage FAQ*.
https://support.mozilla.org/en-US/kb/total-cookie-protection-and-website-breakage-faq

Mullvad VPN AB. (2026, January 13). *The Mullvad Browser hard facts: list of
settings and modifications*. https://mullvad.net/en/browser/hard-facts

ungoogled-software. (n.d.). *ungoogled-chromium*. GitHub.
https://github.com/ungoogled-software/ungoogled-chromium

World Wide Web Consortium. (2025, September 25). *Mitigating Browser
Fingerprinting in Web Specifications* (W3C Group Note).
https://www.w3.org/TR/fingerprinting-guidance/

The proposed exception flow, BR acceptance questions and development order are
project design based on the recorded assignment and the technical
references above; the sources do not certify Nieuw OS.
