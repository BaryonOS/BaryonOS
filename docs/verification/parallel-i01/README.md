# I01 — brongebonden native qsim-build en gerichte controles

28 oktober 2026. Basis `1acf6e8cde2a63849dc49c948ad97ba7f2c39cad`, werkbranch
`codex/parallel-native-qsim-20261028`. De concrete nieuwe bron wordt door
`checks.json` en `build-report.json` op bestands-SHA256 vastgelegd.

## Resultaat op dit checkpoint

De bestaande `qsim-core` is zonder algoritme- of afhankelijkheidswijziging
gelinkt in een zelfstandig `x86_64-unknown-none`-gebruikersprogramma. De
nieuwe adapter begrenst jobs op zes qubits, 64 gates en 4.096 shots. De native
allocator begrenst het totale procesarena op 65.536 bytes. Er zijn uitsluitend
aanroepen naar de bestaande `abi_info`/`yield`-syscalls; geen opslag-, netwerk-,
MMIO- of beheerrechten worden toegevoegd.

**Dit checkpoint bevat geen eigen QEMU-run.** De integrator moet de ELF
native laden, de rapportvelden onafhankelijk vergelijken, de andere
gebruikersprocessen controleren en onverwachte fouten/time-outs afwijzen.
Targetcompilatie wordt hier niet als native uitvoering gerapporteerd.

## Werkelijk uitgevoerde controles

| Controle | Uitkomst |
|---|---|
| Bestaande qsim-core, direct als path dependency getest | 17 Rust-hosttests geslaagd |
| Nieuwe bounded adapter en allocatorrekenregels | 6 Rust-hosttests geslaagd |
| Gedecodeerde-instructiegate met negatieve controles | 3 Python-tests geslaagd |
| Host-Clippy, `-D warnings` | Exit 0 |
| Native-target-Clippy inclusief freestanding probe | Exit 0 |
| Rustfmt | Exit 0 |
| Native targetbuild en ELF-grenzen | Exit 0; drie gescheiden RX/RW/R-segmenten |
| Volledige `.text`-disassembly | 4.096 instructies; vier `int 0x80`-sites; geen x87/MMX/SSE/AVX/opmask |

Exacte opdrachten, exitstatus en loghashes staan in `checks.json`. De
native ELF-hash is
`ee9ec71e1c37719c51ffd92021b62aaa8c9d6303bc92482547d717ac1d3a0481`.
De volledige gedecodeerde instructies en symbolen zijn bewaard in
`disassembly.txt` en `symbols.txt`; binaries en caches staan buiten Git.
De software-floathelpers `__adddf3`, `__muldf3`, `__divdf3` en `__floatdidf`
komen uit de officiële Rust-targetbibliotheek.

## Wat de toekomstige native run controleert

Bell4096 moet exact `[2082, 0, 0, 2014]` geven; een zes-qubit X-circuit moet
alle 4.096 shots op basisindex 63 geven. Daarnaast zijn voor/tijdens
uitvoering annulering, een geïnjecteerde deadline, invoerlimieten en
werkelijke `ResourceExhausted` van de bestaande core ingebouwd. Het
rapportcontract staat in `experiments/quantum/native/README.md`.

Het allocatorgeheugen wordt bewust niet tussentijds hergebruikt: de eindige
proef is één proceslevensduur. Een blijvende jobdienst, IPC-cancelobject,
algemene native klok en fysieke QPU-ondersteuning blijven afzonderlijk werk.

## Ontwikkelgereedschap

De al aanwezige Rust 1.90-hostinstallatie is naar een nieuwe taakmap gekopieerd.
De twee ontbrekende officiële targetarchives zijn met de officiële
manifest-SHA256 gecontroleerd en alleen in die kopie geïnstalleerd. Ook
Debian 12 QEMU 7.2.22, OVMF en de benodigde libraries zijn lokaal uitgepakt;
er is geen systeeminstallatie, bestaande toolchainwijziging of configuratie
van echte apparaten uitgevoerd. Versies, bronnen en pakket-SHA256 staan in
de twee `*-toolchain-source.json`-bestanden.

QEMU 7.2.22 wijkt af van de 10.0-referentie; een run moet daarom expliciet
`pc-q35-7.2` en de echte firmwarehashes vastleggen. De rootintegrator beheert
die profielkeuze en de eigen QEMU-buildmap.

## Score en vervolg

Voorstel voor dit checkpoint: L02 blijft open, +0 punten; er is nog geen
native uitvoeringsbewijs. Root herbeoordeelt het vaste criterium pas na de
gecombineerde native acceptatie, zonder de criteria of gewichten te wijzigen.
Terugname bestaat uit het uitschakelen van de opt-in quantumproef en het
behouden van de ongewijzigde bestaande hostsimulator.
