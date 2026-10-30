# Native desktop-init en proefcontroller

Afzonderlijke ring3-init voor het vaste GUI-manifest, plus een apart beperkt
CPL3-proefproces voor een echte gecontroleerde STOP-uitwisseling.

Contract en opstartmetadata: [NATIVE-DESKTOP-INIT.md](../../../docs/contracts/NATIVE-DESKTOP-INIT.md).
Bouw beide echte scalar-ELFs met de beheerde omgeving via
`python3 sdk/examples/desktop-init/build.py`; JSON geeft
`NOS_DESKTOP_INIT_ELF` en `NOS_DESKTOP_CONTROLLER_ELF`.

Hosttests gebruiken echte capability-engine-envelopes en protocoltestdata;
ze bewijzen geen native hardwarefocus, rendering, opslag of gebruikersbediening.
Native integratie en bijbehorende image-/QMP-bewijzen volgen afzonderlijk.
