# Native compositorcomponent

Echte ring3-server voor één toegewezen Materiaal-surface. De productiecode gebruikt de eigen SDK voor kernel-IPC, monotone tijd en DisplaySource-blits. De dienst heeft geen volume-, sleutel-, netwerk- of rawinputrecht.

Het bootstrap-/SUR1-/CMP1-contract en de open grenzen staan in [NATIVE-COMPOSITOR.md](../../../docs/implementation/NATIVE-COMPOSITOR.md). De eigen Cargo-workspace gebruikt dezelfde gepinde Noto-bitmapasset en gedeelde Materiaal-kleurentokens. Licenties en herkomst staan in [THIRD-PARTY.md](THIRD-PARTY.md).

Bouwen: `python3 sdk/examples/compositor/build.py` vanuit de repository met de beheerde toolomgeving en de ABI1.8 SDK-wrappers. Root ontvangt `NOS_COMPOSITOR_ELF`; er wordt geen kernel of hostdisplay in deze component nagebootst.

De hostscenario's toetsen foutafhandeling en IPC-correlatie met echte capability-engine-envelopes. Hun beeldschermtransport weigert **iedere** write; zij leveren geen native beeldbewijs. De componentcontrole bevat daarnaast een echte scalar `x86_64-unknown-none` ELF-build en controle van de vijf disjuncte LOAD-segmenten. Werkelijke compositor-/init-/input-/Materiaaluitvoering, zichtbare bediening, maximale opstart-/wistijd, herstel en langdurig gebruik wachten op de afzonderlijke integratiegate. Dit is geen volledige desktop- of releaseacceptatie.
