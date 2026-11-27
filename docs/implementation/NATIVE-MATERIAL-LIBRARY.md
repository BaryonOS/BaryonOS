<a id="native-materiaal-naam-collectie-en-geschiedenis"></a>

# Native Materiaal: name, collection and history

Baseline `159ad799d2a94fb6542d60e26ed8bbd22bdbd752`, 2 November 2026.
Implementation for U03; the criterion remains open until native acceptance.
The existing session/privacy/storage contract remains authoritative.

<a id="integratie-stack130"></a>

### STACK130 integration

The later library and PR127-retention share `Layout::for_screen` for
rendering, pointer handling, navigation, caret scrolling and damage. In narrow
windows, the library toolbar moves the editor field; damage must not use the
earlier position from `Layout::new`. The same proportional text metrics
determine both the old and new affected rows, including soft-wrap affinity.
The journal is retained until the entire current frame is acknowledged.
Exact pixel comparison, negative partial-supersede tests and the existing
minimum reduction of 90% in SUR messages remain mandatory without change.
This is a source integration; native runtime acceptance remains separate.

<a id="gebruikersroute-en-visuele-regels"></a>

## User journey and visual rules

Materiaal remains an object with its own content, name and revisions. The library
shows actual local documents, a search field and the four bounded collections
supported by the service. A fresh disk shows a useful empty state.
Sample content is not presented as personal work.

The document window is to use proportional text with one layout for text,
selection, caret, wrapping, pointer handling and vertical navigation. The title,
local location, save status, permissions and history are to occupy fixed, recognizable
positions. Neutral surfaces, a restrained hierarchy and existing system
typography follow QUIET-SESSION-SPEC; no new palette, card grid or decorative
animations. UTF-8 bytes never change to make the display fit. Full shaping,
IME and a native screen reader remain separately unproven.

- F2 edits the name; Enter changes the draft, Ctrl+S explicitly saves.
- Ctrl+F searches locally by name; Escape leaves the field without losing the draft.
- Ctrl+G opens organization and collection management; a persistent change may
  be shown as successful only after a confirmed service operation.
- Alt+H opens history. Restore reads an existing version and creates
  a new revision; it does not replace evidence of the previous revision.

<a id="dienstgrens"></a>

## Service boundary

No new storage format, kernel permission or IPC ABI is introduced. The existing
volume-v7 service enforces domain, object, expected revision and read-only
permissions. SDK wrappers reject mutation during saving, in dirty/conflict/uncertain
states and on an incorrect revision. Each published mutation is sent at most once.
An uncertain outcome closes the connection; the UI must not retry blindly
or display success. A confirmed restore followed by a failed reread is reported
separately and must not save the old draft as the new current content.

<a id="acceptatie"></a>

## Acceptance

Host tests check text metrics, UTF-8 boundaries, pointer/caret, scale, focus,
stale actions, read-only, dirty, conflict, timeout and uncertain mutations.
Native QMP performs creation, typing, renaming, saving, organization, search,
opening, history and restore on a new disposable disk. Entire assigned app images
are compared exactly using the same Rust renderer, without masks.
The existing independent disk decoder then checks actual bytes,
name, collection, revisions, history and canaries. A second fresh VM
on a verified copy of only that test image demonstrates cold
reopening; reading must leave the disk unchanged.

The existing complete clipboard/compositor recovery gate remains separately
mandatory. New host images do not replace native execution, a human task trial,
account authentication or hardware evidence. Score and evidence are reassessed
only after those checks.

<a id="gemeten-native-codebudget"></a>

## Measured native code budget

Byte counts below retain the original period as a thousands separator.

The first linked library version measured 131.177 bytes of RX code in the actual
scalar linker map: 105 bytes above the earlier 128-KiB demo budget, after which
the budget became 144 KiB. A separate `opt-level=z` experiment also exceeded
that earlier budget; thin LTO is not usable with the pinned
prebuilt-core/code-model combination. Both failed experiments remain
diagnostic evidence, not accepted binaries.

The Veld version measured exactly 157.161 bytes of RX code (39 pages),
9.705 bytes above 144 KiB, after which the budget became 160 KiB. The SUR1
reply-wait version (R02, 26 November 2026) measures 164.355 bytes (41 pages),
515 bytes above 160 KiB, against 163.507 bytes on `main` at that time. On the
project owner's decision the app-specific linker script therefore reserves
a fixed **176 KiB RX** from the same address `0x400f0000`. Read-only remains
512 KiB; data/BSS remains 256 KiB;
the three 64-KiB document buffers, entry/startup ABI, stack/guards and kernel quotas
remain unchanged. No W+X, capability, device or kernel ABI expansion is introduced.
The actual ELF is checked again with the same kernel parser for all five segments,
non-overlap, permissions and total frame quotas. This does not justify ignoring
unexpected further growth or failed native acceptance.
