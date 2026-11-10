# Baryon OS app building blocks

This independent workspace contains bounded libraries for user processes.
They do not change the kernel ABI, capability rights or storage format.

| Component | Function |
|---|---|
| [runtime](runtime/README.md) | Allocation-free `no_std` wrappers for the existing int80 ABI, Message320 and typed handles. |
| [catalog-client](catalog-client/README.md) | B01 catalogue client over the runtime, with bounded waiting, canonical replies and no replay after publication. |
| [document-session](document-session/README.md) | Open an existing document, edit locally, save with CAS and explicitly reopen; retain drafts on conflict or uncertainty. |
| [catalog-read](examples/catalog-read/README.md) | Compilable native LIST → GET example using the typed catalogue client. |
| [supervisor](examples/supervisor/README.md) | K05: ring-3 supervisor, restartable worker and client over the spawn/kill/status ABI 1.3; acceptance programs for the `native-supervisor` profile. |

G01/G03 demonstrate safe host validation and native compilation. The separate
G05 suite demonstrates the runtime bridge in the existing native M6c chain.
G07 is a new app building block with host-model tests and native library
compilation; it does not claim additional GUI or app-startup evidence.
Existing G1/Materiaal work remains with its assigned owner.

Use a separate `CARGO_TARGET_DIR` for parallel work. The `document-session`
README describes the public API and the app's explicit recovery choices.
