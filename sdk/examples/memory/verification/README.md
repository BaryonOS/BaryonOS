# Bewijsgrens van deze ontwikkelproeven

`checks.json` bindt vijf echte VM-runs aan één bronhash, de exacte app-ELF en
vijf kernel-ELF's. Iedere submap bewaart de ruwe seriële bytes, het volledige
runrapport, bouwmanifest en gebruikte bouwcommando's. De afzonderlijke
negatieve OOM-VM heeft128MiB; de andere vier hebben256MiB. Dit is geen
productprofielclaim voor8GiB.

De gedeelde integratiebestanden zijn geen onderdeel van deze fixturecommit.
De gebruikte tijdelijke M3-hooks staan in `m3-hooks.patch`; de tijdelijke
Cargo-features staan in `checks.json`. `development_gate.py` en de toen gebruikte
kopie `run_root_helper.py` bewaren de exacte ontwikkelprocedure als bewijsdata.
Deze procedure verwacht de opgenomen hooks in `build/memory-fixture/`, een
schone gedeelde M3/Cargo-basis en bestaande passende QEMU/OVMF-bestanden.
Gebruik de door de hoofdagent geïntegreerde canonieke `run_memory.py` voor
nieuwe hoofdbranchacceptatie. Oude lokale runs vervangen die herhaling niet.

De nulcontrole na opruiming vindt plaats vóór enige nieuwe allocatie en
controleert fysiek ieder vrijgegeven heapframe. Herstel van één4096-bytepagina
in de OOM-proef is al opgenomen in `GROWN_PAGES`, `ZERO_BYTES` en
`CLEANUP_ZERO_BYTES`. `RETRY_ZERO_BYTES` mag niet dubbel worden opgeteld.

Alle getallen zijn daarnaast onafhankelijk van de guest-PASS-regels uit de
ruwe uitvoer geparseerd met negentien unieke geordende velden, gecontroleerde
formules en werkelijke artifact-/serialhashes. Dat is een aanvullende
harnascontrole, geen onafhankelijke bronreview of formeel bewijs.
