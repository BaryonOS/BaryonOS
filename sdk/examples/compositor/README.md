# Native compositor component

Actual ring3 server for one assigned Materiaal surface. Production code uses the project's SDK for kernel IPC, monotonic time and DisplaySource blits. The service has no volume, key, network or raw-input rights.

The bootstrap/SUR1/CMP1 contract and remaining limits are in [NATIVE-COMPOSITOR.md](../../../docs/implementation/NATIVE-COMPOSITOR.md). Its separate Cargo workspace uses the same pinned Noto bitmap asset and shared Materiaal colour tokens. Licences and provenance are in [THIRD-PARTY.md](THIRD-PARTY.md).

Build with `python3 sdk/examples/compositor/build.py` from the repository, using the managed tool environment and ABI1.8 SDK wrappers. The lead agent receives `NOS_COMPOSITOR_ELF`; this component does not simulate a kernel or host display.

Host scenarios test error handling and IPC correlation using actual capability-engine envelopes. Their display transport rejects **every** write; they provide no native visual evidence. Component validation also includes an actual scalar `x86_64-unknown-none` ELF build and inspection of the five disjoint LOAD segments. Actual compositor/init/input/Materiaal execution, visible interaction, maximum startup/clear time, recovery and sustained use await the separate integration gate. This is not complete desktop or release acceptance.
