# Native desktop init and test controller

Separate ring3 init for the fixed GUI manifest, plus a separate restricted
CPL3 test process for an actual controlled STOP exchange.

Contract and startup metadata: [NATIVE-DESKTOP-INIT.md](../../../docs/contracts/NATIVE-DESKTOP-INIT.md).
Build both actual scalar ELFs in the managed environment using
`python3 sdk/examples/desktop-init/build.py`; JSON provides
`NOS_DESKTOP_INIT_ELF` and `NOS_DESKTOP_CONTROLLER_ELF`.

Host tests use actual capability-engine envelopes and protocol test data;
they do not demonstrate native hardware focus, rendering, storage or user
interaction. Native integration and associated image/QMP evidence follow separately.
