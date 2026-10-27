# Ontwerpbesluiten
Status revisie 5: onderstaande principes zijn projectbesluiten. Concrete keuzes voor de eerste bouwfase staan in implementation/START-CODING.md en bijbehorende specificaties. Bouwuitvoerbaarheid is nog niet getest.

| ID | Besluit | Reden | Herzien als |
|---|---|---|---|
| ADR-001 | Eigen kernel; geen bestaande kernel als basis | Expliciete gebruikerswens | Simon bewust de projectscope verandert |
| ADR-002 | Rust no_std + minimaal assembly | Kleine expliciete onveilige grens | Target/tooling dit aantoonbaar verhindert |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulatie baseline | Controleerbare eerste testomgeving | Praktische toolchainbeperking wordt aangetoond |
| ADR-004 | Kleine capability-gebaseerde microkernel als richting | Autoriteit expliciet en TCB klein houden | Prototypes aantoonbare problemen met complexiteit/latency tonen |
| ADR-005 | Standaarden en gecontroleerde bibliotheken toestaan | Zelf crypto/alle tools bouwen vergroot risico en scope | Herkomst/licentie/onderhoud onacceptabel is |
| ADR-006 | Lokaal account, geen externe OS-telemetrie, OS-data alleen apparaat/eigen NAS op eigen LAN | Expliciete gebruikerscorrectie | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-007 | PQC-voorbereiding vanaf ontwerp; integratie pas met echte use case | Protocol- en sleutelmigratie voorkomen | Relevante standaarden/implementaties wijzigen |
| ADR-008, herzien | Product-/interactieonderzoek parallel vanaf M0; echte systeemintegratie volgt bewezen kernfuncties | Originaliteit en hoogwaardige UX moeten het systeem mede sturen | Bijstelling op experimenten, geen stille reductie tot standaarddesktop |
| ADR-009 | Geen kernelbrede claims over formeel bewijs | Eigen code erft geen seL4-bewijs | Afgebakend daadwerkelijk geverifieerd resultaat bestaat |
| ADR-010 | AI-inferentie uitsluitend op apparaat; geen netwerkrecht of cloudfallback | Expliciete gebruikerscorrectie | Alleen na expliciete gewijzigde gebruikersopdracht |
| ADR-011 | Installeerbare derdenapps met gescheiden bestands-, NAS- en netwerkrechten | Toegankelijk ecosysteem zonder impliciet vertrouwen | Acceptatietests tonen ontbrekende grens |
| ADR-012 | Quantum als derde onderzoeksspoor vanaf M0: PQC, lokale QPU-uitvoering en hybride systeemport | Oorspronkelijke ambitie expliciet behouden | Bijstelling alleen met onderbouwd onderzoek; privacycontract blijft gelden |

## Gekozen uitvoeringsbesluiten — 26 oktober 2026

| ID | Besluit | Bewijsgrens / herziening |
|---|---|---|
| ADR-013 | Eigen UEFI-loader met r-efi 7.1.0; Rust 1.90.0; x86_64/QEMU TCG | Versies onderzocht; combinatie nog niet gebouwd. Exact bootcontract in BOOT-AND-TOOLCHAIN.md |
| ADR-014 | Eerste kernel: vaste ELF-layout en gedeelde BootInfo; daarna begrensde processen, IPC en capabilities | Details en negatieve tests in KERNEL-AND-SECURITY-CONTRACT.md; geen isolatiebewijs vóór uitvoering |
| ADR-015 | Materiaal als eerste productrichting; P2 lokale HTML/CSS/JS-proef | Vergelijking uitgevoerd, geen gebruikersproef. Herzie op concrete gebruiksobservaties |
| ADR-016 | Q1 safe Rust statevector-core met qsim-v0, maximaal 12 qubits, hostadapter apart | Analytische fixtures gecontroleerd; geen Rust-/QPU-/OS-portbewijs |
| ADR-017 | Later ML-KEM-768 en ML-DSA-65 via beoordeelde implementaties/protocolbinding | Geen eigen cryptoprimitives, geen volledige bootketenclaim |
| ADR-018 | EU-01 t/m EU-08 als technische acceptatie-eisen; toepasselijkheid wetgeving per release | EUROPE-AND-RELEASE.md bevat conditionele matrix, geen conformiteitsclaim |

## Template voor nieuw besluit
ID, datum, status, context, alternatieven, besluit, onderbouwing/bronnen, gevolgen, security-invariant, testbewijs, migratie/rollback en eigenaar. Schrijf wat de keuze zou weerleggen; vermijd besluiten zonder toetsbare aannames.

## Uitvoeringsbesluiten — 27 oktober 2026

**ADR-019 — Windows-ontwikkelpakket en alleen-lezen virtio-ESP.** Rust1.90.0/MSVC gebruikt de bestaande Visual Studio2026/SDK. QEMU11.1.0 en het bijpassende CODE/VARS-paar komen uit dezelfde vastgelegde Windows-package; hashes staan in docs/verification/toolchain-provenance.json. Dit is de toegestane ontwikkelvariant, niet de byte-identieke referentiebuild. De oorspronkelijke IDE-drive gaf vóór loaderuitvoering `Block node is read-only`; de ESP gebruikt daarom `if=virtio,readonly=on`. Iedere run krijgt een eigen ESP-/VARS-kopie, zonder netwerk of hostdisk. Werkelijke cold-boots en foutproeven staan in docs/verification/boot-runs.json. Herzie bij andere firmware/platformen; verwijdering van de afzonderlijke toolmap herstelt de host zonder globale PATH-wijziging.

**ADR-020 — Vaste kernelspan32–48MiB en BootInfo1.1.** In de echte OVMF-kaart overlapt het oorspronkelijke interval2–18MiB ACPI NVS en BootServicesData. AllocatePages wees de reservering af met EFI_NOT_FOUND; diagnose in lokaal build/run/smoke-fe68c261dd/result.json. Het profiel gebruikt nu `[0x02000000,0x03000000)`, dezelfde16MiB. Loader, parser, linker en fixtures zijn samen gewijzigd; de header is nog128bytes met dezelfde offsets, maar vereist expliciet ABI1.1. ABI1.0 wordt afgewezen. Er wordt geen bezet firmwaregeheugen hergebruikt. De volledige acceptatiereeks met deze waarden slaagt. Terugdraaien vereist gezamenlijk aanpassen van beide binaries/validator/linker en een aantoonbaar passende firmwarekaart; nooit alleen een adresfallback invoeren.

**ADR-021 — Statische GOT in .data.** De compiler produceert vijf statisch opgeloste pointers. Die staan expliciet in de bestaande RW-.data-outputsectie; er komt geen runtime-relocatieverwerker of extra segment. De buildinspectie blijft dynamische metadata, relocaties en onbekende allocatable secties weigeren. De drie ELF-laadsegmenten en hashes staan in docs/verification/boot-manifest.json. M1 controleert segmentintentie; echte W^X-afdwinging blijft M2.

**ADR-022 — Voortgang per stap naar GitHub.** De gebruiker wil het project bij verbruikslimieten volledig kunnen hervatten. Broncode, lockfiles, compacte bewijsrapporten en open punten gaan daarom na elke afgebakende stap en vóór lange controles naar de privéwerkbranch. Een checkpoint mag onvoltooide controles bevatten mits expliciet gemarkeerd. Codespaces is niet vereist voor deze bewaarroutine; een Codespace is niet gestart. Main blijft de startspecificatie totdat de concept-PR is beoordeeld.

**ADR-023 — Alleen WB-capabele normale ConventionalMemory, cachebits gecombineerd.** De eerste M2-run faalde vóór KS-02 doordat OVMF `0x0f` meldt en de allocator uitsluitend exact WB accepteerde. UEFI-attributen beschrijven mogelijkheden; UC/WC/WT/WB mogen samen voorkomen. De allocator vereist WB, staat uitsluitend deze vier cachebits toe en weigert overige/bijzondere attributen. Bron: TianoCore/EDK II, `edk2-stable202508`, [UefiSpec.h, EFI_MEMORY_DESCRIPTOR en cachebits](https://github.com/tianocore/edk2/blob/edk2-stable202508/MdePkg/Include/Uefi/UefiSpec.h), gecontroleerd 27 oktober 2026. De huidige package is daarmee niet als exact die firmwareversie geïdentificeerd. Regressie test zowel `0x0f` als geweigerde special/unknown-bits; echte M2-poort slaagt. Andere platformprofielen vereisen opnieuw beoordeling van geheugenattributen; geen stil reclaim van Loader/BootServices/runtime/ACPI-geheugen.

**ADR-024 — Statische2048-mappingarena en één tijdelijk scratchslot.** Iedere procescontext heeft de contractcapaciteit2048 mappings; grote metadata staat statisch buiten de16KiB-kernelstack. PID is monotoneu64; interne ASID is afzonderlijk begrensd. Allocatorzeroing/tabeltoegang gebruikt een tijdelijk supervisor/NX-slot, met invalidatie vóór gebruik en na clear. Een writable scratchalias naar een executable fysiek frame wordt geweigerd; readonly input heeft een aparte RO/NX-helper. Geen willekeurige identitymap van vrije RAM/gaten/MMIO. Bewijs: kernelbron ed09818, volle M2/M3-gate, echte67-userpage/BSS-zero-regressie en per-processcleanupbalans. Herzie bij SMP/DMA/nieuwe mapping-API; de serialisatie geldt alleen in het huidige één-CPU/één-threadprofiel. Herstel via een consistente eerdere bronversie in een afzonderlijke checkout, zonder huidige wijzigingen te wissen.

**ADR-025 — Safe vaste capability-engine met afzonderlijke usercopygrens.** `capability-core` is eigen dependencyvrije no_std-code zonder unsafe/heap; de kernel bewaakt userpointers, syscallvolgorde en320-byte-kopieën. Engine-receive commit pas na volledige outputvalidatie, daarna volgt de in dit profiel onfeilbare bytecopy. Rootautoriteit komt uitsluitend uit het statische testmanifest, niet uit een algemene userspace-objectlookup. Revoke trekt afstammelingen in; exit ook eigen/issued autoriteit. Gesloten objecttombstones blijven bestaan tot laatste ref. Hosttests vergelijken de volledige Engine-toestand; kernelchecksums zijn diagnostiek. Bewijs:21coretests,alle acht echte M4-cases en onafhankelijke gerichte review. Herzie vóór threads/sharedmemory/DMA/dynamische applicatie- of objecttypen; geen extrapolatie naar zulke garanties. Geen boot-ABI-wijziging of hostconfigmigratie.
