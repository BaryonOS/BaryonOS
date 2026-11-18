# Repository baseline for follow-up research — phase 1 recorded

**Historical English equivalent.** Research date: **2 November 2026**. Source family **RB-01**. This translation preserves the observations and limits of the [archived Dutch original](repository-baseline.md), whose SHA-256 is `56f825756dd90f672367dafce57b45f67d7a536f1703a2dc6eddbb0bfb543054`. The original remains unchanged. This is a compact record of the earlier read-only analysis, not a new security audit or native rerun. The scores and implementation observations below belong to the recorded source versions, not the current product checkpoint. Relative links are retained from the historical record; their current targets may have changed.

## Verified source states

| Source | Recorded state |
| --- | --- |
| Local main clone and live main | `e08fb0e48c1bb0781e98357f588c2f73b6ae8500`; 5,359 tracked files inventoried; stored roadmap 34/100 |
| Newer development branch, checked live | `codex/mature-native-session-20261102`, pinned to `1f0dd9c93959e3b1335656da1e548525a288133e`; stored roadmap 41/100 |
| Local PC preview | Older source `7648a8f`; separate QEMU bundle, not the current development branch |
| This research branch | `codex/os-research-phases-2-4-20261102`, branched from main; research and documentation status only |

The local development ref `e5ffc3b` proved to be behind during phase 1. The relevant current sources were therefore read through the GitHub connector at exactly `1f0dd9c`; no checkout was switched and no existing work was overwritten. Score 41 was not copied to the research documentation branch based on main.

## Architecture and product

[REPOSITORY] The project had its own Rust UEFI loader and x86_64 `no_std` kernel, capability-oriented process/IPC boundaries, separate system services, its own native SDK and a graphics layer. No existing OS kernel supplied the foundation. The repository contained multiple Cargo workspaces/lockfiles, Python/QEMU harnesses and Node host models. Sources: [workspace](../../../Cargo.toml), [kernel contract](../../implementation/KERNEL-AND-SECURITY-CONTRACT.md).

[REPOSITORY] Materiaal was the selected document-oriented product model: objects/collections, interchangeable tools, data location, bounded rights and recovery. The browser prototype used HTML/CSS/JavaScript with a reducer, mock broker, fixtures and temporary browser storage. Navigation: Mijn materiaal (My material), Gereedschap (Tools), Apparaten (Devices), Assistent (Assistant). NAS, installation and AI were simulated in this layer. Sources: [product specification](../../implementation/PRODUCT-SPEC.md), [prototype](../../../ui/prototype/README.md).

[REPOSITORY] The pinned development branch added an actual native desktop: Materiaal and Rekenmachine (Calculator) as isolated processes, two windows, focus, dragging, a dock and compositor recovery that preserved application work. U01 acceptance and partial N01 network evidence were stored; a 192-byte UTF-8 text clipboard was integrated later. U02 performance, actual account authentication and full product acceptance remained open. [Checkpoint](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/PROJECT-STATE.md), [U01 evidence](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/docs/verification/mature-session-20261102/U01-ACCEPTANCE.md).

## Constraints for external design proposals

- Two fixed windows; no general dynamic application/window list, resizing, snapping or virtual-desktop controls were found. [Scene](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/sdk/examples/window-session/src/scene.rs).
- Native session search looked up a fixed Materiaal reference; the browser prototype filtered titles/collections/types. No integrated local content/application/action/settings index was present.
- Window session: scale 1 or 2, at least 800×500 logical pixels and at most 1920×1080 physical pixels. Mobile adaptability and general HiDPI coverage had not been demonstrated.
- The Open Sans atlas had limited character coverage; complex shaping, IME, bidirectional text and general font fallback were absent. Native semantic roles/focus existed at application level; the OS-wide assistive-technology chain remained open. [Graphics](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/README.md), [native frontend](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/native/README.md).
- Browser and native palettes differed; spacing/colours came partly from CSS, partly from Rust and fixed scene values. P2 was not the final visual identity. [Tokens](../../../ui/prototype/tokens.css), [native theme](https://github.com/simonvanieperen/nieuw-os/blob/1f0dd9c93959e3b1335656da1e548525a288133e/ui/graphics/src/theme.rs).
- Capability core: 16 processes, 256 slots per process, 320-byte IPC. Volume: 16 objects in total, 64 KiB/document, 32 UTF-8 name bytes, two historical revisions, four collections per domain. The older DocumentSession catalogue route had smaller content limits than the newer volume route. [Core](../../../crates/capability-core/src/lib.rs), [volume](../../../services/volume/src/lib.rs), [DocumentSession](../../../sdk/document-session/src/lib.rs).
- Native TLS/provider/trusted time, full NAS synchronisation, broad application compatibility, local AI inference, physical drivers and daily administration remained separate development tasks. A directory or host model was not an integrated OS feature.

## Design analysis, not a user finding

[ANALYSE] The strongest existing coherence lay in material identity, bounded rights, storage status and recovery. The newer application-oriented Start/dock route needed to be made compatible with those concepts. Ordinary desktop conventions were recognisable; no one-to-one copy had been established. An original-looking pattern did not prove usability. The original analysis marker is retained as a research identifier.

Research-relevant gaps: uniform settings, a system-wide rights inventory, a notification/focus model, general discovery, extended document management, accessible scaling/language and explicit migration. These were integrations found missing in code or left open in documentation, not demonstrated user priorities.

## Verification boundary

During phase 1, the two existing browser-model test suites and `node --check ui/prototype/app.js` passed, as did `python -B tools/progress.py --check` on main. The Git diff remained empty apart from the pre-existing untracked `.github/github-app.yml`, which was not read or changed. No fresh native build, QEMU run, screen-reader trial, contrast measurement or human research was performed. That documentation step repeated only the relevant document/source checks; historical OS tests were not presented as newly executed.
