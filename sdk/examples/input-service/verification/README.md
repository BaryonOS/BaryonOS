# Componentbewijs voor native inputdaemon

`checks.json` bindt de echte targetELFs, dertien hostscenario’s, twee Clippy-
controles, formattering en linked disassemblycontrole aan bron-/dependencyhashes.
Alle exacte Cargo-/bouwcommando’s staan onder `logs/`. Het bouwscript in de
package is de reproduceerbare entrypoint. De volledige lokale disassemblies
blijven onder de genoemde eigen buildpaden bewaard met hashes; objdump/nm
kunnen rechtstreeks op de geregistreerde executables herhaald worden.

De root-SDKpatch was alleen tijdelijk tijdens de targetcontrole toegepast en
is daarna hersteld. `sdk-wrappers-used.patch` is een bewijsopname van die
expliciet toegestane dependency, geen onderdeel van deze daemonimplementatie.
De hoofdagent integreert die wrappers en bootgrants afzonderlijk.

`native_runtime_performed` is false. De RawInput-hostfixture levert uitsluitend
testdata voor sequencing. De echte executable gebruikt InputClient met
NativeTransport en native SDK IPC. Hosttests of een getypecheckte ELF bewijzen
geen PS/2-hardware, kernelafzenderherkomst, native driverReady, daadwerkelijke
registryontvangst of werkende Materiaalbediening. Die volledige native keten
blijft open tot de hoofdagent de eigen releaseimage werkelijk uitvoert.
