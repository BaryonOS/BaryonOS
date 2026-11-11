# Desktop, taskbar and search — design

Original design: 30 October 2026 (ADR-058). Status as of 9 November 2026:
**design** (§§1–5); **host-rendered implementation** (§7, `ui/graphics` and
`ui/native`, PR #214). There is no new native QEMU evidence for the Horizon
appearance and no measured usability study. The accepted progress remains
**39/100**; no additional criterion is accepted by this change.
Design source: [DECISIONS.md](../DECISIONS.md), ADR-058.
Sign-in and users: [PROFILES-AND-USERS.md](PROFILES-AND-USERS.md).

## 1. Goal

Anyone who knows Windows, macOS or a common Linux desktop can use Nieuw OS
right away: start apps, search, arrange windows, find files again,
change settings and shut down, without explanation. On top of that Nieuw OS offers
things that familiar platforms do not offer this way by default (§4). Those additions
must never replace or get in the way of the familiar path.

Rules:

- **Familiar first.** A standard action uses the place, the gesture and
  the shortcut that consumers expect.
- **Own house style.** No copied icons, images, fonts,
  brand names or trade dress. The existing Materiaal tokens (mineral
  background, deep green tones, clear typography) are the starting point.
- **Rights remain rights.** A trusted-looking window or a familiar
  place grants no capability. Permission dialogs are drawn by the trusted
  shell in a layer that apps cannot draw over. Password, PIN
  and administrator password are entered only on a surface of the
  session service, reached via the lock screen or Ctrl+Alt+Delete; the
  shell and apps never receive those keystrokes
  ([PROFILES-AND-USERS.md](PROFILES-AND-USERS.md) §4, invariant 8). An app
  can imitate pixels; that is why the system never asks for a
  secret outside that route.

## 2. Components and chosen conventions

| Component | Behaviour | Choice where platforms differ |
|---|---|---|
| Desktop | Background; optional shortcuts; right-click: new document, change background, display settings | Empty desktop with only a background by default |
| Orbit (dock) | Zenit button (own horizon brand mark, no "Start"), Signaal (search), pinned and open apps with indicator, system tray, clock/date, notifications | Compact floating capsule at the bottom, centered; the location stays familiar |
| Zenit (launcher) | Search field at the top (typing directly searches), pinned apps, all apps, recent files, user and power menu | Opens with the OS key or the brand mark in Orbit |
| Search | Apps, settings and own files, indexed locally; results appear directly while typing | No internet search by default; only if the user turns it on and chooses a search service |
| Windows | Title bar with minimize, maximize and close; dragging; resizing via the edges; double-click on the title bar maximizes | Buttons on the right |
| Window arranging | Dragging to an edge or corner, or OS key+arrow, puts the window in a half or quarter | — |
| Switching | Alt+Tab with thumbnails; click on a taskbar button | — |
| System tray | Network, sound, battery, privacy indicator (§4); click opens quick settings (Wi-Fi, volume, brightness, do not disturb) | — |
| Notifications | Notification center from clock/date; do not disturb | — |
| Files | Files app with sidebar (Recent, Desktop, Documents, Pictures, Downloads, NAS/devices) and address bar; Materiaal provides history, rights and storage status | Known folders as fixed entry points |
| Settings | Categories: System, Devices, Network and internet, Personalization, Apps, Accounts, Time and language, Accessibility, Privacy and security, Update | Search field in Settings |
| Recycle bin | Deleting goes to the recycle bin first; emptying it is permanent | — |
| Screenshot | Print Screen and OS key+Shift+S | — |

### Default keyboard shortcuts

| Key | Action |
|---|---|
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / cut / paste |
| Ctrl+Z / Ctrl+Y | Undo / redo |
| Ctrl+S, Ctrl+A, Ctrl+F, Ctrl+P | Save, select all, find, print |
| Alt+Tab, Alt+F4 | Switch, close window |
| OS key | Zenit with Signaal |
| OS key+E / +I / +L / +D | Files / Settings / lock / show desktop |
| Ctrl+Alt+Delete | Trusted system screen (lock, switch, sign out, task manager) |
| Ctrl+Shift+Esc | Task manager |

"OS key" means the key with the logo, also called the Super
key. On Mac keyboards Cmd serves as the OS key.

## 3. Current native base and what is missing

| Exists (native, tested in QEMU) | Missing for this shell |
|---|---|
| Ring-3 compositor with one assigned surface for one app, focus and tile blit ([NATIVE-COMPOSITOR.md](NATIVE-COMPOSITOR.md)) | Multiple clients and surfaces, window management (move, resize, stack, arrange), a protected system layer, taskbar and start menu as trusted shell |
| Native input service with PS/2 source and focus route via the compositor ([NATIVE-INPUT-DAEMON.md](NATIVE-INPUT-DAEMON.md)) | Visible mouse pointer, interrupt-driven input, shortcut handling by the shell, secure attention sequence to the session service |
| Materiaal and document session ([NATIVE-MATERIAAL.md](NATIVE-MATERIAAL.md)) | Files app with folder view, search index |
| Multiple ring-3 processes ([K05](K05-PROCESS-MANAGEMENT.md)) | Generic starting/stopping of apps from the start menu (FR-K02, FR-A rows) |
| UEFI framebuffer, fixed 800×600 ([DISPLAY-INPUT-NATIVE-PROBE.md](DISPLAY-INPUT-NATIVE-PROBE.md)) | Higher resolutions, scale factor, a graphics driver |

## 4. Better than familiar (additions)

These additions come on top of §2, never in place of it:

- **Local assistant in search:** questions about self-chosen files,
  entirely on the device without network right (FR-L01/L02).
- **Privacy indicator:** the system tray shows which app is currently using network,
  microphone, camera or location; one click shows or revokes the right.
- **Clear permission dialogs:** who, what, for how long, in plain language,
  drawn by the trusted shell.
- **History for every file:** restore earlier versions from the
  Files app (Materiaal revisions).
- **Continue working on a second own device** via the own NAS.
- **Finish:** smooth, short animations that can be turned off; no waiting for
  screens that visibly build up.

## 5. Steps

Each step is only done with native QEMU evidence, including failure cases. A
browser mock or screen design is product evidence, not system evidence.

| Step | Result | Depends on | Acceptance |
|---|---|---|---|
| D0 | Controls usable (= step 0 of PROFILES-AND-USERS): pointer, keyboard, complete frames, IRQ input | Compositor, input | See there |
| D1 | Screen design in own house style for desktop, taskbar, start menu, windows, lock and sign-in screen; clickable prototype, labelled as simulation | ADR-058 | Task trial with people who are not administrators: start an app, find a file again, arrange a window, shut down without explanation |
| D2 | Multiple clients and surfaces in the compositor; window management: title bar, move, resize, stack, arrange, Alt+Tab | D0 | Two apps at once; focus, overlap and closing; an app cannot move or draw over another window |
| D3 | Taskbar, start menu and power menu as trusted shell; starting apps from the menu | D2, generic process management | Starting, closing, crash of an app leaves the shell standing; crash of the shell recovers without session loss |
| D4 | Search across apps, settings and own files, locally | D3, catalog, PROFILES-AND-USERS step 3 | Only files of the signed-in user; no network traffic without explicit choice |
| D5 | Files app and Settings app | D3, PROFILES-AND-USERS steps 2 and 6 | FR-U and FR-S03 acceptance |
| D6 | Higher resolution and scale, then graphics driver | Driver work | Separate contracts |

## 6. Evidence boundary

Sections 1–5 describe the intended interaction design. Section 7 records a
host-rendered desktop implementation. Its fixtures and unit tests do not
demonstrate native Horizon execution, consumer usability, or feature parity
with another operating system. Familiarity requires measured user-task trials
(U06).

## 7. Appearance 2030 — "Horizon" (5 November 2026)

Status: **host-rendered** via the existing desktop fixtures in `ui/native`
(768×536, 1280×720, 1920×1080); no new native QEMU evidence and no measured
usability trial.

User decision (Simon, 5 November 2026): no "Start" button or an icon
reminiscent of an existing OS logo, and no shape that feels like a
pimped-up classic taskbar. The **location** of the controls stays familiar
(bottom, search, apps, sign out); shape, name and lighting are our own.

Design language "Horizon":

- **Light as material.** Background: a dark sky above the glowing
  rim of a planet, with atmospheric light, an iris haze and sparse
  stars. The rim is computed with 1/16-pixel precision, so that it also stays sharp on
  FHD, QHD and 4K screens. Integers only, so every
  tile renders identically.
- **Floating glass with a light rim.** Surfaces are dark translucent glass
  with a fine edge and a highlight from above (`rim`). There are no white
  form cards.
- **One command dock, not a taskbar.** A centered capsule contains the
  original Zenit horizon mark, compact Signaal search, pinned Nebula and
  power. The privacy light is visual status only. The running-app light
  strip is **planned**, not currently visible: opening Nebula switches
  to the Material page, where Orbit is not rendered.
- **Privacy as light, not as a pop-up.** A calm mint-green glow
  "Lokaal" (local) in the dock. It is not a control and asks nothing. It says
  only what is true in this VM session: your workspace is on this device.
  Mint means "stays with you" everywhere. The full indicator from §4 waits for
  a capability source.
- **Centered composition** for large screens: "Verder waar je was" (pick up where you left off) as
  the first view, instead of a panel at the top left.

Deliberately not yet done:

- a clock or date: the UI has no time source;
- fake apps in the overview;
- animation;
- scale above 2×: 8K needs a 4× glyph atlas, and the renderer currently knows
  three font sizes at 1× and 2×;
- large display typography: that requires an extra atlas size.

### Names (user decision 5 November 2026, family "Kosmos")

Only the visible names change; code identifiers and document names
(`Materiaal`, `nieuw_os_materiaal`) stay in place to avoid churn.

| Known as | Nieuw OS | Status |
|---|---|---|
| Desktop / start screen | **Horizon** | Visible in the VM shell |
| Taskbar / dock | **Orbit** | Visible (compact capsule) |
| Start / launcher | **Zenit** | Visible (brand mark, label) |
| Explorer / files (was Materiaal) | **Nebula** | Visible in shell and app |
| Search | **Signaal** | Visible (button and search panel) |
| Settings | **Kern** | No app yet |
| Store | **Haven** | No app yet |

### Direction: what an OS of 2030 must give the consumer

| What people miss now | Answer in Nieuw OS | Building block |
|---|---|---|
| Privacy costs dozens of settings and cookie-like pop-ups | Privacy is on by default; rights are chosen once per **object and consequence** when installing or pairing, silent afterwards; a permanent light status in the dock instead of notifications | Capabilities, §4 privacy indicator |
| Work is scattered across apps, cloud and folders | One command field for apps, material and actions, indexed locally; "Verder waar je was" as the starting point | D4, Materiaal |
| "Where is my version from yesterday?" | History per file, restore from Files | Materiaal revisions |
| A second device means a cloud account | Continue working via the own NAS, end-to-end encrypted | NAS sync |
| AI assistants send data to the cloud | Local assistant in the command field, without network right | FR-L01/L02 |
| High-resolution screens look blurry or too small | Vector-like, integer-exact rendering; scale 1×/2× now, 3×/4× (8K) as the next step | Glyph atlas, D6 |
| Updates and errors feel scary | Undoing updates and clear recovery routes | Signed updates, recovery |

"Feels futuristic" and ease of use remain hypotheses until the task trial
(U06).
