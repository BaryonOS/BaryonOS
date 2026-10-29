# Ontwerpbesluiten

## ADR-031 — Native I/O-onzekerheid zonder observerpanic

**Datum/status/eigenaar:** 28 oktober 2026; geïmplementeerd en geaccepteerd op
bron 88e6c04. Native builder A01, integratie root, AI-reviews R08/R09 en A04.

**Context en besluit:** de dienst kende Unknown al, maar de M6c-observer eiste
bij een voorbereide mutatie altijd Ok. Bind de eerste onzekere reply aan een
werkelijk aan de dienst teruggegeven terminale muterende syscallfout; Pending
is geen fout. Latch de I/O-tellingen, commit geen kandidaat, tel geen ACK en
controleer dat beide appdomeinen daarna geen nieuwe I/O veroorzaken. Validatie
en autorisatie blijven vóór de gesloten diensttoestand. Alleen volledige koude
recovery bepaalt de werkelijk opgeslagen versie.

**Alternatieven en gevolgen:** de succesasserties verwijderen zou fouten maskeren;
automatische write-replay kan een onzekere mutatie verdubbelen. Het gekozen pad
houdt normale WRITE+FLUSH-voorwaarden streng en verandert geen capability,
syscall, wire-/journalformaat, dienstimplementatie of externe dependency.
Testinjecties lopen via bestaande driversnapshot-, deadline- en resetpaden.
Geen automatische dienstherstart; historische M6b-observer apart behouden.

**Bewijs en herziening:** de volledige 35-groepen/31-VM-gate en M6b-regressie
(18 groepen/14 VM) slagen, met beide captures en afzonderlijke bewijsaudits.
30 Rust/48 Python-tests voor M6c, 22 Rust/30 Python-tests voor M6b; alle exit 0.
Exacte scope en bron: `verification/M6D-ACCEPTANCE.md`; S07 levert 13% → 14%.
Contract en primaire VIRTIO-bron:
`implementation/M6D-IO-UNCERTAINTY.md`. Geen fysieke devicefout, stroomuitval,
menselijke audit of formeel bewijs. Herzie bij algemene servicesupervisie,
parallelle mutaties, nieuwe I/O-foutmodellen of andere hardware.

## ADR-032 — Vaste 100-puntenroadmap met verplichte herbeoordeling

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard en uitgevoerd op expliciete
gebruikersopdracht. De hoofdagent bezit de gedeelde voortgangsbestanden.

**Besluit:** `docs/progress.json` bevat vaste, binaire acceptatiecriteria met
gewichten die samen 100 vormen. De generator maakt de README, criteriatabel
en lokale SVG. 0% is nog niet gerealiseerd; 100% is een volwaardig zelfstandig
dagelijks OS op aantoonbaar ondersteunde hardware. De eigen kernel blijft
verplicht. De beginscore van de M1–M6c-bewijsstand is 13%.

**Alternatieven/gevolgen:** commits, regels code, testaantallen of M-nummers
meten geen productvoltooiing. Daarom leveren hostprototypes of alleen plannen
geen native punten op. Iedere opdracht actualiseert beoordeling, bron en datum;
een score mag gelijk blijven of dalen. Gewichten zijn een expliciete
planningskeuze, geen wetenschappelijke meting of resterende-tijdsraming.

**Bewijs en herziening:** `tools/progress.py --check` controleert som, unieke
criteria, bestaande bewijsverwijzingen en gelijkheid van gegenereerde weergaven.
Het beoordeelt de bewijsinhoud niet en voert geen native tests uit. Wijzigingen
aan eindscope/gewichten vereisen rubricversie, reden en vergelijking met de oude
score. Zie `PROGRESS.md`; nooit de lat verlagen om een hoger getal te krijgen.

## ADR-030 — Native labels en een catalogus per appdomein

**Datum/status:** 28 oktober 2026; geïmplementeerd en native eindgate geslaagd
op broncommit bc4cd104, basis bb22fdb. Exact contract: implementation/M6C-NAMED-CATALOG.md.
Op deze historische M6c-uitvoeringsbron heeft het native `Unknown`-foutpad een
bekende, ongerepareerde observerpanic, ook met `milestone-m6c`. De afzonderlijke
reparatiebron en haar afzonderlijke acceptatie staan bovenaan in ADR-031.

**Context en besluit:** afgeronde M6b heeft anonieme objectslots. Materiaal heeft
benoemde objecten en list/open nodig. Voeg een aparte dependencyvrije catalog-core
met expliciete IPC v4/journal v2 toe. Namen zijn UTF-8-labels, geen autoriteit of
pad; object-ID en endpoint/caller blijven de toegangsgrens. LIST retourneert één
begrensde snapshot van uitsluitend eigen live objecten met eigen revisiesom.
PUT/RENAME zijn één record met naam/inhoud onder één CAS-revisie. RENAME behoudt
inhoud; WRITE- en FLUSH-completion gaan vóór ACK. Het servicecontract sluit bij
`Unknown` ook LIST af; dit foutpad is alleen via host-ABI getest.

**Alternatieven/gevolgen:** stil hergebruik van v3/v1-padding zou oude readers
misleiden; een losse naamtabel vereist meerdere atomische writes. Automatische
migratie is niet nodig voor deze synthetische proef: v1 weigert vóór READY en de
M6b-reader blijft behouden. Duplicaatlabels zijn toegestaan; geen directoryboom,
normalisatie, encryptie of fysieke secure erase toegevoegd. Kleinere grenzen zijn
proefbudgetten, geen vaste consumentenproductkeuze.

**Bewijs en herziening:** de finale M6c-gate/capture slaagt met 19 groepen en
15 VM-invocaties: 14 M6c plus één gewone M1. M6b-regressie slaagt met 18 groepen
en 14 VM-invocaties op dezelfde fingerprint. De 30 coretests, 33 harnastests en
15 host-ABI-adapterproeven behouden hun eigen scope. De native observer in
`kernel/src/m2/m3/m6c.rs` kan bij WRITE-/FLUSH-fouten of time-outs panikeren
vóór de client `Unknown` ontvangt; gecontroleerde native foutafhandeling was
op die bron niet gerealiseerd. Een afzonderlijke AI-audit
controleert alle 141 bronbestanden, 130 geregistreerde artifacthashes en acht
volumes, inclusief twee externe rename-kills en zes koude recoveries. Details:
`verification/M6C-ACCEPTANCE.md`; geen menselijke audit of formeel bewijs.
De nieuwe kernelmodule bevat een expliciete fixtureobserver plus de bestaande blokbroker; het
catalogusbeleid draait in ring 3. Herstel van het bekende `Unknown`-defect
vereist een afzonderlijke native reparatie met WRITE-/FLUSH-fout- en
time-outinjectie, controle van het werkelijk ontvangen clientantwoord en
nieuwe brongebonden native verificatie. Herzie formaat/migratie vóór echt gebruikersdata,
meer slots, grotere documenten of compositorintegratie. Terugdraaien kan via de
bestaande M6b-build en zijn eigen volumes; geen bestaande image overschrijven.

**Bronnen:** huidige M6B-OBJECT-STORAGE.md/PROJECT-STATE.md op bb22fdb; Rust
bare-metal targetdocumentatie in het M6c-contract. Dit is een projectbesluit.

## ADR-028 — Bewijs per eigenschap en standaard hybride NAS-transport

**Datum/status:** 27 oktober 2026; ontwerp aanvaard en N02-hostproef getest vanaf `ece94a5`. **Context:** de gebruikersblauwdruk vraagt formele verificatie, kleine TCB, capabilities, geheugenbescherming en PQC. De richting past bij het project; de absolute productclaims en het eigen KDF-/tunnelvoorstel zijn niet onderbouwd.

**Besluit:** eigen Rust-kern behouden, formele eigenschappen met expliciete aannames en implementatiebinding uitwerken, CHERI als aparte platformproef. Asymmetrische/PQC-code blijft buiten `kernel/`. Voor de eerste uitvoerbare NAS-transportproef TLS 1.3 met uitsluitend de gestandaardiseerde groep X25519MLKEM768 (RFC 10024), wederzijdse certificaatcontrole/pinning en bestaande peergebonden rechten. Geen eigen combiner/recordcrypto. De optionele-PSK-framing uit het aangeleverde fragment is ambigu. Een klassieke-only peer krijgt geen fallback in dit proefprofiel.

**Alternatieven/gevolgen:** onmiddellijk een ongeteste native crate toevoegen past niet bij de huidige directe-rustc-build, vaste ELF-limieten, kleine IPC, ontbrekende entropycapability en uitgeschakelde SIMD. Een hele kernel vervangen door seL4 is geen gegeven opdracht. De begrensde hostproef geeft werkelijk protocolbewijs zonder een native port te claimen. De proef gebruikt klassieke Ed25519-authenticatie; geen volledige postkwantum-authenticatie-/bootketenclaim. LAN/WAN-labels in loopbacktests zijn expliciete harnasaannames.

**Invariant:** toestemming vóór verbinden; echte peer-authenticatie vóór data; actuele bevoegdheid bij iedere applicatiewrite/response; close/error trekt transportfeit in. Fact bindt aan app, peerpin-generatie en voor internet de intrekkingsgeneratie; oude geauthenticeerde verbindingen herleven niet na hernieuwde toestemming. Intrekking werkt bij de volgende gecontroleerde handeling, geen proactieve idle-socketpush. Client-only documentkeys en het bestaande beheerderverwijderrecht blijven gelden. Een geldig ondertekend pakket verleent op zichzelf geen capabilities. Eenmalig verleende beperkte autoriteit mag automatisch gebruikt worden zonder herhaalde prompts.

**Bewijs/status:** geïntegreerd 124/124 tests (85 service, 39 UI), beide demo's exit 0, 20 syntaxchecks en diffcontrole geslaagd. Echte negatieve TLS-handshakes, geauthenticeerde kwaadaardige serverantwoorden, resourcegrenzen, intrekking en regrant getest. Twee AI-reviews en gerichte onafhankelijke hercontrole van de generatiecorrectie. Commando's/bronhashes/resultaten in `verification/pqc-transport/`. Het eerste 16-policytestcheckpoint is historisch bewaard en niet bij de 124 opgeteld. Geen formeel bewijs of menselijke audit.

**Bronnen:** RFC 10024; seL4 verified configurations/Proofcraft nieuws 2026; CHERI FAQ/CHERI-D; NIST FIPS 203/204/205. Precieze URLs, correcties en native taken staan in `implementation/VERIFIABLE-SECURITY-AND-PQC.md`.

**Herziening/rollback:** herzie bij incompatibele standaarden, library-/bewijsproblemen, native port of nieuwe productinstructie. N02 is een afzonderlijke werkbranch boven PR #7; verwijderen ervan vereist geen schijf-, sleutel- of firmwaremigratie. De nieuwe proef opent uitsluitend loopback. Bestaande N01/M6a-code en bewijsarchieven behouden.

## ADR-027 — Internet en versleutelde persoonlijke NAS-ruimte

**Datum/status:** 27 oktober 2026; aanvaard op expliciete gebruikersopdracht, met geteste hostproef N01 en browsermock. Native integratie staat open. **Eigenaar:** hoofdagent voor contract/integratie; afzonderlijke service- en productbuilders.

**Context:** de gebruiker wil internet, een NAS-rol binnen hetzelfde OS en opslagruimte aanbieden aan vrienden/familie. De host moet zijn ruimte kunnen terugnemen zonder toestemming van de gast, terwijl de inhoud voor hem versleuteld blijft. Dit herziet ADR-006 en de eerdere beperking tot eigen NAS/eigen LAN.

**Alternatieven:** alleen eigen LAN handhaven voldoet niet aan de nieuwe opdracht; een leverancierscloud of beheerderssleutel voor gastinhoud schendt de privacygrens; gastconsent als vereiste voor verwijderen ontneemt de hardware-eigenaar zijn beheerrecht.

**Besluit/invariant:** client-only inhoudssleutels; versleuteling van documentnaam en inhoud; namespace/object/revisie geauthenticeerd. NAS-beheer beheert quota, intrekking en verwijdering maar krijgt geen decryptiesleutel. Revoke/evict invalideert oude sessies en lopende uploads; tombstones verhinderen herleving. Internet is een afzonderlijk apprecht; NAS-internet is expliciet peergebonden, lokale AI blijft netwerkloos. Lokale NAS-beheerrol staat los van een internet-datakoppeling; extern beheer wordt niet stilzwijgend toegevoegd. Een routeplanner kiest automatisch geverifieerd LAN of eerder toegestaan direct internet naar dezelfde peer, zonder herhaalde consentvragen of nieuwe autoriteit. Privacy en gebruiksgemak zijn samen vereist; tijdelijke hostproefcaps zijn geen productkeuze.

**Uitvoering:** dependencyvrije Node-hostproef in services/nas/, zonder sockets of echte NAS. Eigen synthetische ciphertextstore met één schrijver en atomische snapshots. Browserflows Mijn NAS / Ruimte ontvangen / Ruimte aanbieden zijn mocks. Deze ontwikkelmiddelen worden geen verplichte runtime van het eigen OS.

**Testbewijs:** gezamenlijke suite 89/89 (50 service, 39 UI) en eindige demo exit0; bronhashes/commando's in verification/nas/report.json. Echte childproceskills na bevestigde opslagmutaties, geen fysieke stroomuitvaltest. Vier concrete reviewbevindingen zijn hersteld: noncebudget bij staging, tijdelijke ciphertextsnapshots bij wissen, race bij writerlockherstel en ongewenst beheerrecht via internet.

**Grenzen/gevolgen:** hostidentiteit en transportfacts zijn synthetisch; geen TLS-/native netwerkbewijs. Herstelde inhoudssleutel kan alleen lezen, niet veilig blijven schrijven op meerdere apparaten. Browserinternet is in N01 appgebonden; bestemmingsgrants voor gewone apps/downloaders volgen nog. Geen volledige snapshot-rollbackbescherming, veilige fysieke erase of plug-and-play-NAT-claim. Native clientdistributie/key lifecycle, netwerkcapabilities, beheeropslag, onboarding en menselijke audit blijven afzonderlijke integratievoorwaarden.

**Bronnen:** Node.js v24.22.1 Crypto (https://nodejs.org/docs/latest-v24.x/api/crypto.html), RFC 8446 (https://www.rfc-editor.org/rfc/rfc8446.html) en NIST SP 800-207 (https://doi.org/10.6028/NIST.SP.800-207), geraadpleegd 27 oktober 2026. Functionele keuze en adminverwijderrecht zijn projectbesluiten. Exact contract: implementation/NAS-SHARING-AND-INTERNET.md en LOCAL-PRIVACY-CONTRACT.md revisie 3.

**Migratie/rollback/herziening:** geen bestaande echte NAS, router, kernel-ABI of M6a-image wordt gewijzigd. Bewaar oudere M1–M6a-resultaten. N01 kan als afzonderlijke featurecommit worden teruggedraaid; geen destructieve reset van gebruikerswerk. Herzie bij bewijs van key-/nonceproblemen, multi-writerintegratie, andere transport-/opslaglaag of een nieuwe gebruikersinstructie. Oude LAN-only teksten zijn op de actieve leesroutes bijgewerkt.
De oorspronkelijke revisie-5-principes hieronder zijn behouden als projectbesluiten. Uitvoeringsbesluiten vanaf ADR-019 beschrijven de gebouwde richting; actuele M1–M5-testuitkomsten en grenzen staan in PROJECT-STATE.md.

| ID | Besluit | Reden | Herzien als |
|---|---|---|---|
| ADR-001 | Eigen kernel; geen bestaande kernel als basis | Expliciete gebruikerswens | Simon bewust de projectscope verandert |
| ADR-002 | Rust no_std + minimaal assembly | Kleine expliciete onveilige grens | Target/tooling dit aantoonbaar verhindert |
| ADR-003 | x86_64 QEMU q35, UEFI, software-emulatie baseline | Controleerbare eerste testomgeving | Praktische toolchainbeperking wordt aangetoond |
| ADR-004 | Kleine capability-gebaseerde microkernel als richting | Autoriteit expliciet en TCB klein houden | Prototypes aantoonbare problemen met complexiteit/latency tonen |
| ADR-005 | Standaarden en gecontroleerde bibliotheken toestaan | Zelf crypto/alle tools bouwen vergroot risico en scope | Herkomst/licentie/onderhoud onacceptabel is |
| ADR-006, herzien door ADR-027 | Lokaal account, geen OS-telemetrie, lokale data of E2EE naar gekoppelde eigen/gedeelde persoonlijke NAS; internet per peer expliciet | Expliciete gebruikersuitbreiding 27 oktober 2026 | Alleen na expliciete gewijzigde gebruikersopdracht |
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

**ADR-026 — M5: geïsoleerde READ-dienst met vertrouwd virtio-transport.** De eerste systeemdienst draait als eigen Rust-programma in ring 3. De kernel bewaakt de getypeerde READ-capability, tickets, usercopy, PCI/MMIO en een vaste vijfpagina-DMA-pool. Het device op `00:05.0` gebruikt één splitqueue van acht plaatsen en maximaal één gelijktijdige sectorread. De dienst kan geen fysieke adressen, descriptors, PCI-configuratie of MMIO kiezen. Het protocol en de 17 foutcases staan vóór uitvoering vast in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). Alleen eigen lokale crates zijn toegevoegd; geen nieuwe externe dependency.

**Status:** aanvaard en geïmplementeerd voor het vaste M5-QEMU-profiel. **Alternatieven:** alle dienstlogica in de kernel (grotere vertrouwde basis), of directe userspace-DMA (vereist aanvullend isolatieontwerp, buiten M5). **Eigenaar:** hoofdagent voor integratie en besluitbeheer; `kernel_m2` voor transport en mappinggrens.

**Onderbouwing/bronnen:**

- OASIS Open, *[Virtual I/O Device (VIRTIO) Version 1.2](https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html)*, Committee Specification 01, 1 juli 2022: §2.4 (resetbevestiging), §2.7 en §2.7.8 (splitqueue en used-entry zonder epoch), §4.1 (PCI-transport) en §5.2 (blokdevice). Queuegrootte, één gelijktijdige read en de gepinde pool zijn projectkeuzes.
- Intel Corporation, *[Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide, Part 1](https://www.intel.sg/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)*, Order No. 253668-060US, september 2016: §4.1.4 en §4.5 (CPUID/MAXPHYADDR en gereserveerde fysieke adresbits), §§11.12.2–11.12.3 (IA32_PAT en selectie met PAT/PCD/PWT). `map_mmio` controleert de CPU-adresbreedte en vereist UC in PAT-entry 3 vóór de 4KiB-mapping met PAT=0, PCD=1 en PWT=1; de firmware-PAT blijft ongewijzigd. Bronnen gecontroleerd op 27 oktober 2026.
De vertrouwde basis omvat het kerneltransport en het protocolconforme QEMU-device/backend. M5 bevat geen IOMMU: CPU-isolatie en begrensde descriptoropbouw bewijzen geen containment van kwaadwillige DMA. De pool blijft gepind tot VM-stop; hergebruik vereist completion of bevestigde reset. Zonder resetack blijft zij in quarantaine en het device offline. Een used-entry heeft geen epoch; softwaretickets vervangen geen hardware-resetgarantie. Herzie dit besluit vóór directe userdriver-DMA, SMP, fysieke hardware of een andere devicebackend.

De eerste scope is alleen lezen op een synthetische 64KiB-wegwerpimage. Schrijven, persistentie en herstel blijven M6. Herstart gebruikt een begrensde testcoördinator en echte nieuwe dienst-PID/epoch plus IPC-capoverdracht; geen algemene spawn-API. De client blijft leven. Het externe testorakel controleert sectorchecksums op twee verschillende images bij dezelfde kernel en controleert dat alle imagebytes ongewijzigd blijven. Uitgevoerde resultaten en open punten staan uitsluitend in PROJECT-STATE.md en verification/m5/. Herstel: een afzonderlijke checkout van M1–M4 op `c630c59`; geen hostdisk- of firmwaremigratie.


## ADR-029 — Native objectdienst met revisies en begrensd journal

**Datum/status/eigenaar:** 27 oktober 2026, geïmplementeerd en volledige native gate geslaagd binnen het begrensde QEMU-profiel. Het native `Unknown`-foutpad heeft een bekende, ongerepareerde observerpanic. Hoofdagent integreert, afzonderlijke builders en AI-reviewers controleren service, broker en harnas.

**Context en besluit:** M6a bewaart één vaste waarde. M6b voegt een echte ring-3-objectdienst toe boven dezelfde begrensde READ/WRITE/FLUSH-ABI: vier objecten in twee endpointgebonden domeinen, GET/PUT/DELETE, maximaal 128 bytes en 32 append-only records. De limieten zijn proefgrenzen. De object-core is eigen dependencyvrije safe Rust; er worden geen nieuwe externe crates toegevoegd. De kernel houdt de bestaande DMA-broker plus een expliciete testobserver; objectpolicy draait in de dienst.

**Alternatieven en gevolgen:** een volledig bestandssysteem met namen, garbage collection en encryptie ineens toevoegen maakt de herstel- en bevoegdheidsgrens veel groter. Een extra hoststore levert geen native vooruitgang. Dit begrensde native fundament maakt meerdere objecten en blijvende verwijdering nu toetsbaar, met expliciete latere uitbreidingen voor ruimtehergebruik, namen, crypto en de desktop. Geen migratie van bestaande M6a-images; M6b gebruikt uitsluitend eigen wegwerpvolumes.

**Invariant:** de ontvangende endpointcapability bindt het domein. Geen appvelden als blok- of domeinautoriteit. Mutaties vergelijken de actuele revisie, en alleen een echte WRITE-completion gevolgd door FLUSH-completion mag een succesvolle reply opleveren. Een tombstone behoudt de revisie, zodat oude verzoeken geen verwijdering ongedaan maken. Corruptie geeft geen stilzwijgende terugval. Het servicecontract vereist dat onzekere mutaties `Unknown` opleveren en verdere geldige dienstoperaties blokkeren zonder automatische replay. Dit `Unknown`-contract is alleen op host-ABI-niveau getest en is voor native WRITE-/FLUSH-fouten niet gerealiseerd door de bekende observerpanic. Full verschijnt vóór muterende I/O. Verwijderen is logisch; oude payloadbytes blijven in het journal en zijn niet cryptografisch beschermd.

**Onderbouwing en bewijs:** eigen contract `implementation/M6B-OBJECT-STORAGE.md`, eerdere virtio/capability-ABI uit ADR-026/M6a; wire- en journalindeling zijn interne projectbesluiten. De volledige eindgate op `1e61a20` slaagt: 18 groepen/14 echte VM-runs, 22 Rust-hosttests en 26 harnastests. Daaronder vallen zes koude recoveries, uitputting, corruptie en twee echte externe kills. Capture en een aparte audit van 121 artifacthashes/zeven schijfvolumes slagen; een extra Python-parser controleert onafhankelijk de echte schijfbytes zonder de Rust-journalcode te delen. Zeven host-ABI-adapterproeven controleren `Unknown`-uitkomsten. Native WRITE-/FLUSH-fouten kunnen via `sent()` in `kernel/src/m2/m3/m6b.rs` een kernel panic veroorzaken: de observer vergelijkt de reply eerst met een verwachte `Ok`-reply. Dit defect is niet gerepareerd; gecontroleerde end-to-end-afhandeling als `Unknown` ontbreekt ook in `milestone-m6b`. De native gate bevat geen devicefoutinjectie voor dit pad. Exacte bron, regressies en bewijsgrenzen staan in `PROJECT-STATE.md` en `verification/m6b/`.

**Herziening en rollback:** een herziening van het native `Unknown`-pad vereist een native reparatie, gerichte WRITE-/FLUSH-devicefoutinjectie, controle van het daadwerkelijke clientantwoord en nieuwe native verificatie met bijbehorende bronbinding. Herzie ook bij meer objecten, compaction, meerdere schrijvers, sleutels, native UI of andere devices. M6b heeft een eigen feature/ELF/volumeformaat; bestaande M1–M6a-profielen blijven selecteerbaar. Een aparte werkbranch maakt terugname mogelijk zonder hostschijven of echte gebruikersdata te wijzigen. Tests in QEMU bewijzen geen fysieke stroomuitval, IOMMU-containment, formele correctheid of productierijpheid.

Aanvulling 28 oktober 2026 op ADR-029: de terminale WRITE-/FLUSH-observerfout
is op `592dccf` hersteld en native getest met twee statussnapshotinjecties;
de historische beweringen hierboven gelden voor hun oude bron. Verder herstel
van lokale poll-LIMIT, echte brokerdeadline en resetquarantaine is de actieve
vervolgstap K08; uitgevoerde resultaten blijven in PROJECT-STATE.md.

Aanvulling 29 oktober 2026 op ADR-029 (fixturebudget, geen productbeleid): de
virtio-completiedeadline van de fixturebrokers in `kernel/src/m2/m3/m6c.rs` en
`kernel/src/m2/m3/volume.rs` gaat van 64 naar 500 ticks (5 s bij 100 Hz),
omdat een FLUSH een host-`fdatasync` is die onder host-last 0,16–0,31 s meet
en twee CI-runs vóór de foutinjectie een niet-`Ok`-antwoord gaven dat bij een
gemiste deadline past (niet gereproduceerd). Beide brokers rapporteren de
maximale completielatentie en het aantal verstreken deadlines als diagnostiek,
ook op faalpaden. PR #26 verhoogde dezelfde constante eerder van 20 naar 64
zonder ADR; deze regel legt het budget en zijn motivering nu wel vast.
Herzieningscriterium: zodra een productdienst een eigen I/O-deadline krijgt
(K08) wordt dit budget opnieuw vastgesteld op basis van de gerapporteerde
latenties; de 30 s die gangbare besturingssystemen als blokapparaat-timeout
hanteren is een referentie, geen bron voor dit project.

## ADR-033 — opt-in native grafische bootstrap vóór productintegratie

Datum 28 oktober 2026, aanvaard als begrensde ontwikkelstap; root integreert,
G09 beheert core/loader/adapter. Het browserprototype geeft geen bewijs voor
native scherm- en invoerondersteuning. Daarom komt eerst een afzonderlijke
G0-feature met gevalideerde GOP-metadata, supervisor framebuffer en begrensde
PS/2-polling. Alternatieven zijn een volledige compositor/driver/IPC-broker
ineens bouwen (te grote eerste hardwarestap), of een browser op de host als
native presenteren (geen voortgang op eigen hardwaregrens).

Invariant: opt-in ABI 1.2 met vaste descriptor in de behouden BootInfo-pagina,
geen firmwarecalls na EBS, geen framebufferalias naar RAM/user/executable,
strikt begrensde RW/NX/UC-devicepagina's en safe `no_std`-render/invoercore.
De kernelweergave is tijdelijke diagnostiek, geen app-, object- of opslagpolicy.
Het scherm labelt de proef. Geen rechten voor willekeurige apps en geen nieuwe
extern gepubliceerde productie-ABI.

Contract/bronnen/acceptatie: [GRAPHICS-G0.md](implementation/GRAPHICS-G0.md).
Testbewijs wordt afzonderlijk opgeslagen; dit besluit is geen boottest.
Herzie vóór ring-3-compositor, IRQ-invoer, DMA/GPU, SMP, Unicode/IME of echte
hardware. Herstel via aparte checkout `7b4945f`; standaard BootInfo 1.1 blijft
selecteerbaar, geen echte schijf- of firmwaremigratie.


## ADR-034 — Native SDK en begrensde quantumport als afzonderlijke profielen

Datum 28 oktober 2026; geïmplementeerd als ontwikkelprofielen, eindacceptatie
op de gecombineerde bron nog uit te voeren. Root beheert ABI/bootconfig;
G01–G03 bouwen SDK/client, I01–I04 de port en controles.

De SDK hergebruikt de bestaande capability-ABI en 320-byte IPC-envelop.
Canonicalisatie, endpointrechten, antwoordherkomst en één uitstaand verzoek
zijn expliciet begrensd. Alleen EAGAIN vóór enqueue mag opnieuw worden
geprobeerd. Na enqueue, Unknown, ongeldige reply of deadline sluit de sessie
zonder automatische mutatieherhaling. De catalogus bepaalt revisies/namen;
de kernel krijgt geen app- of documentbeleid. De bestaande M6c-probe kan het
SDK-transport selecteren, terwijl ruwe negatieve ABI-proeven hun oude pad
behouden. Een apart native voorbeeld gebruikt de echte catalogusclient;
compilatie is nog geen bewijs van algemene appstart.

De quantumport gebruikt de ongewijzigde eigen qsim-core met maximaal zes
qubits, 64 gates, 4096 shots en een vaste arena van 64 KiB per proces. Een
monotone allocator vermijdt onbewezen hergebruik; de proef beëindigt de
processen waarna de bestaande kernelcleanup de pagina's terugneemt. Software-
float uit de gepinde Rust-toolchain maakt de eerste port mogelijk zonder
schedulerwijziging voor FPU/SIMD-context. Een disassemblygate controleert het
concrete ELF inclusief instructieprefixes; deze controle is geen formeel
bewijs. Een nieuw opt-in profiel bevat twee userprocessen en uitsluitend een
vaste kerneltestobserver. Het profiel is exclusief ten opzichte van M4/G0.

De alternatieven — nieuwe syscalls, onbegrensde allocatie, FPU-state support
of een volledige jobdienst tegelijk — vergroten deze eerste wijzigingsgrens.
Gevolg: native referentieberekeningen en resourcegrenzen zijn nu uitvoerbaar,
maar willekeurige jobaanvoer/resultaten/annulering buiten het gecompileerde
proefprogramma blijven een eigen volgende stap. De Control-fixtures bewijzen
geen externe cancel of klokdeadline. L02 krijgt daarom nog geen punt.

Eigen contracten: SDK README/ABI, `implementation/QUANTUM-SPEC.md` en de
bestaande capability/M3-contracten. Gerichte native uitvoering, exacte
opdrachten, hashes, reviewbevindingen en grenzen worden afzonderlijk bewaard
in `verification/parallel-*`. Herzie vóór dynamische apps, meer qubits,
allocatiehergebruik, FPU/SIMD, SMP of een echte jobdienst. Beide profielen zijn
opt-in; herstel door hun eerdere bron in een aparte checkout te bouwen.
Geen migratie van echte data, nieuwe externe crate of fysieke hardwareactie.


### ADR-034: afzonderlijke invarianten en weerlegging

**SDK — security-invariant:** gewone apps verkrijgen alleen reeds toegekende
procesgebonden endpointrechten. Elke reply wordt canoniek en op herkomst
gecontroleerd. Na enqueue en verlies van zekerheid blijft een mutatie zonder
automatische replay; een ongeldige reply geeft geen extra rechten of succes.
**Weerlegd als:** dezelfde mutatie na Unknown/verloren reply opnieuw wordt
verstuurd, een vreemde afzender wordt geaccepteerd of een ongeldig capability-
bericht als geldig doorgaat. **Eigenaar:** G01–G03 voor SDK, root voor ABI en
integratie. **Testbewijs:** parallel-g01/g03 en parallel-final-sdk-m6c (deze
laatste toetst de runtimebridge, niet de losse catalogusvoorbeeldapp).
**Migratie/rollback:** bestaande ABI en journal blijven gelijk; SDKselectie is
opt-in, terugname via eerdere bron in aparte checkout zonder datamigratie.

**Quantum — security-invariant:** elk proefproces heeft private mappings en
maximaal 64 KiB arena; input blijft binnen 6 qubits / 64 gates / 4096 shots. SIMD/FPU en
vreemde syscall/hardware-instructies zijn niet toegestaan in het geselecteerde
scalarartifact. Simulatorbeleid blijft userspace; opruiming gebruikt echte
procesframes. **Weerlegd als:** grenzen kunnen worden overschreden, processen
elkaars geheugen bereiken, ongewenste instructies door de artifactgate komen
of cleanup toegewezen frames achterlaat. **Eigenaar:** I01–I04 voor port/gate,
root voor de testobserver en integratie. **Testbewijs:** parallel-i04,
parallel-final-qsim en parallel-final-m3. Dit is geen formeel bewijs.
**Migratie/rollback:** exclusief opt-in profiel; eerdere bron afzonderlijk
bouwen, zonder opgeslagen gebruikersdata of hardwarefirmware te wijzigen.


## ADR-035 — Security Floor blijft actief bij vrijwillig minder privacy

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard als product- en
architectuurinvariant, nog niet volledig native gerealiseerd. Hoofdagent bewaakt
integratie; security-eigenaars toetsen de enforcementgrenzen.

**Context:** Nieuw OS moet privacybeschermende standaarden hebben, maar de
gebruiker mag bewust online diensten, telemetrie, microfoon, cloudsync of
andere gegevensdeling toestaan. Een dergelijke keuze mag niet veranderen in
een algemene “trusted app”- of “minder veilige modus”.

**Besluit:** privacykeuzes verruimen alleen expliciete capabilities en
gegevensstromen. Kernel-/procesisolatie, capabilitycontrole, geheugen- en
sleutelisolatie, pakket-/update-authenticiteit, bescherming van niet-gedeelde
data en toekomstige DMA/IOMMU-grenzen blijven buiten die privacykeuze.

**Security-invariant:** toestemming vergroot functionaliteit, niet vertrouwen.
Een internetgrant alleen geeft geen document-, key-, camera-, microfoon-,
NAS-, proces- of kernelrecht.

**Weerlegd als:** een privacytoggle, providerkoppeling, telemetry-opt-in of
focus/gamingmodus een bredere privilegeklasse activeert of bestaande
isolatiechecks omzeilt.

**Bewijs/migratie:** toekomstige native LP-/capabilitytests; geen nieuw
acceptatiepunt door dit document. Bestaande grants blijven semantisch gelijk.
Rollback: herstel dit ADR en de gekoppelde contracten gezamenlijk; geen
datamigratie in deze documentatiestap.

## ADR-036 — Externe provider mag alleen opaque ciphertextopslag zijn

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard architectuurdoel, nog geen
native provideradapter. Opslag-/securityeigenaar beheert formaat en keygrens.

**Context:** de eerdere revisie verbood leverancierscloud volledig. De gebruiker
wil desgewenst bestaande algemene opslag zoals OneDrive/Google Drive kunnen
benutten zonder die provider leesrecht op de opgeslagen inhoud te geven.

**Alternatieven:** absolute provideruitsluiting beperkt keuze; provider-native
encryptie/key escrow verschuift vertrouwen naar de provider; zelfgemaakte
cryptografie is onaanvaardbaar.

**Besluit:** een expliciet gekozen externe provider mag uitsluitend opaque,
client-side versleutelde Nieuw-OS-objecten opslaan. De provider krijgt geen
inhoudssleutel, documentplaintext, gevoelige manifestplaintext, lokale zoekindex
of AI-context. Geen stille providerfallback. Het OS mag een eigen versieerbaar
container-/objectformaat definiëren boven onderhouden standaardcryptografie.

**Security-invariant:** bezit van alleen providerstate/credentials levert geen
geaccepteerde documentplaintext op binnen het gedocumenteerde threat model.

**Weerlegd als:** providercredentials of opgeslagen bytes zonder een geldige
Nieuw-OS-key voldoende zijn voor plaintext; een provideradapter andere lokale
objecten/keys kan lezen; of een outage stil een andere provider activeert.

**Bewijs/migratie:** [implementation/ENCRYPTED-OBJECT-STORAGE.md](implementation/ENCRYPTED-OBJECT-STORAGE.md)
definieert EO-acceptatie. De bestaande NAS-route blijft geldig. Geen echte
provideraccount, productiekey of gebruikersdata wordt door dit besluit
geconfigureerd.

## ADR-037 — Eigen premium native productiviteitsfamilie, geen Office-ripoff

**Datum/status/eigenaar:** 28 oktober 2026; productbesluit voor P3/P4/M7,
nog niet native gerealiseerd. Product-/UI-eigenaar beheert designsysteem;
graphics-eigenaar beheert later renderer/compositorgrenzen.

**Context:** privacyvriendelijke alternatieven mogen niet aanvoelen als
verouderde systeemsoftware. Tegelijk zou een directe kopie van Office,
Windows of macOS de eigen productidentiteit ondermijnen.

**Besluit:** Materiaal blijft het objectmodel. Eigen tekst-, tabel-,
presentatie-, PDF/media- en latere productiviteitstools delen één designsysteem,
contextuele gereedschappen, geschiedenis, delen, herstel, motion en
toegankelijkheid. Geen permanente Office-ribbonkopie als basis. Een gedeelde
native GPU-geaccelereerde UI-/renderlaag is het langetermijndoel zodra de
graphicsbasis dat toelaat; exacte renderer/API blijft een later technisch
besluit. DOCX/XLSX/PPTX/PDF zijn interoperabiliteitsdoelen die per feature
moeten worden getest.

**Productinvariant:** herkenbare basishandelingen blijven voorspelbaar; de
eigenheid komt uit Materiaal, objectcontinuïteit, contextuele tools en
capabilitybewust delen, niet uit onbekende bediening omwille van originaliteit.

**Weerlegd als:** kernapps eindigen als inconsistente losstaande webviews,
de interface structureel een bestaande suite kopieert, of privacy alleen via
beheerachtige UX bruikbaar is.

**Bewijs/migratie:** PX-acceptatie in PRODUCT-SPEC.md en latere gebruikers-/
native tests. Het huidige P2-browserprototype blijft ontwerpgereedschap en
wordt niet door dit ADR tot productruntime verklaard.

## ADR-038 — Privacyuitleg is optioneel; capabilitybeslissingen blijven echt

**Datum/status/eigenaar:** 28 oktober 2026; aanvaard UX-/securitycontract,
nog niet native afgedwongen.

**Context:** begrijpelijke voorbeelden en kritische vragen kunnen privacykeuzes
duidelijker maken, maar herhaalde waarschuwingen kunnen de ervaring verstoren.
De gebruiker wil die privacy-popups volledig kunnen uitschakelen.

**Besluit:** de UI scheidt (a) noodzakelijke beslissingen over nieuwe
capabilities van (b) educatieve privacyuitleg. Educatieve uitleg kan globaal
of per app uit. Een reeds geldige grant wordt niet bij iedere normale
handeling opnieuw gevraagd. Focus/gaming onderdrukt niet-urgente educatieve
meldingen; een bewust gestarte nieuwe gevoelige functie mag nog wel een
compacte noodzakelijke grantvraag tonen.

**Security-invariant:** het uitschakelen van privacyuitleg wijzigt geen
capabilityset en creëert geen impliciete “allow all”.

**Weerlegd als:** dezelfde statewijziging zowel de uitleg als toegangsrechten
uitzet/verruimt, of het systeem op basis van “uitleg uit” een nog niet verleende
capability als toegestaan behandelt.

**Bewijs/migratie:** LP-29/PX-04 en toekomstige native UI-/brokerproeven. Bestaande
grants migreren niet; alleen de presentatievoorkeur wordt afzonderlijk bewaard.

## ADR-039 — DocumentSession native controleren via de bestaande catalogus

**Datum:** 28 oktober 2026.  
**Status:** implementatie geïntegreerd in de auditbranch; host-/crossbuildbewijs en een
actuele twee-VM normal/koude-recovery-proef zijn uitgevoerd, maar dit verandert
geen bestaand roadmapcriterium.  
**Eigenaar:** app/SDK voor de gebruikersproef en library, runtime voor de
acceptatiepoort, root voor selectie en integratie.

**Context:** de bestaande M6c-probe gebruikte de catalogus via directe helpers,
terwijl de echte `DocumentSession`-library al bestaat. De integratiestap moet
die echte library in ring 3 gebruiken zonder kernelobserver, capabilityrechten,
IPC-envelop of opslagformaat te verruimen.

**Alternatieven:** (1) de directe catalogushelpers blijven gebruiken, waardoor
de appbibliotheek niet native wordt getoetst; (2) een afzonderlijke user-ELF met
gekopieerde cataloguscode bouwen, wat duplicatie en bewijsoppervlak vergroot;
(3) de bestaande M6c-userflow opt-in de echte library laten gebruiken en de
bestaande catalogusmodules linken. Gekozen is optie 3.

**Besluit/implementatie:** `DocumentSession<NativeTransport>` draait opt-in in
de bestaande M6c-gebruikersprocessen. `NOS_DOCUMENT_SESSION=1` vereist
`NOS_SDK_RUNTIME=1` en is beperkt tot normal/recover. Openen, lokaal bewerken,
CAS-save, opnieuw openen en LIST blijven userspace. `list()` houdt één eigenaar
van de endpoints; service en typed client delen de bestaande catalogusmodules.

**Security-invariant:** de route gebruikt uitsluitend reeds toegekende
procesgebonden endpoints. Replies blijven canoniek gevalideerd; LIST mag lokaal
concept/conflict/onzekerheid niet wissen en een gesloten sessie niet heropenen.
Onzekere saves worden niet automatisch herhaald. Syscall-, IPC-, geheugen- en
capabilitygrenzen worden niet verruimd. ADR-035 (Security Floor) blijft volledig
van toepassing.

**Onderbouwing/bronnen:** `docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md`,
`sdk/document-session/README.md`, `docs/verification/parallel-g11/`,
`docs/verification/parallel-i13/` en `docs/verification/parallel-i15/`.
Voor de afzonderlijke netwerkparser gelden RFC 768, RFC 791 en RFC 1071, zoals
vastgelegd in `services/net-protocol/README.md`.

**Testbewijs:** de repositorybrede audit-CI voert DocumentSession-hosttests,
SDK-/compilerbindingcontroles, native M6c-build, een baseline M6c-QEMU-smoke en
twee afzonderlijke DocumentSession-VM's (normal en koude recovery) uit.
Historische bewijsbundels blijven aan hun oorspronkelijke bron gekoppeld.

**Weerlegd als:** de proef nieuwe rechten nodig heeft, een draft via LIST
verliest, een onzekere save herhaalt, een gesloten endpoint opnieuw bruikbaar
maakt, de observer/oracle moet versoepelen of een andere binary als documentapp
kan meetellen.

**Migratie/rollback:** standaardprofielen behouden hun bestaande selectie. De
opt-in kan worden uitgezet; er is geen migratie van echte gebruikersdata,
firmware of fysieke apparaten. De headless documentproef is geen Materiaal-GUI,
algemene appstarter, grotere-documentondersteuning of ruimtehergebruik.

## ADR-040 — Copy-on-write volume (M6e) in plaats van append-only journal

**Datum:** 28 oktober 2026. M6b/M6c/M6d blijven ongewijzigd en afzonderlijk
bouwbaar.

**Status (gescheiden):** *ontwerp* geaccepteerd voor het opt-in profiel
`native-volume`. *Implementatie*: `services/volume`, de native dienst/app in
`sdk/examples/volume-docs`, de SDK-blok-API en kernelprofiel `native-volume` op
PR #47. *Testbewijs*: zie hieronder; GitHub Actions moet op iedere nieuwe
PR-head opnieuw groen zijn. *Onbekend/niet getest*: fysieke schijven en
stroomuitval, authenticatie/encryptie, back-up/export en grotere volumes.

**Onderbouwing/bronnen:** OASIS, *Virtual I/O Device (VIRTIO) Version 1.2*,
Committee Specification 01, 1 juli 2022, §5.2 Block Device, §5.2.6.2:
een write is stabiel wanneer na de voltooide write een `VIRTIO_BLK_T_FLUSH`
wordt verstuurd en zelf voltooid is
(https://docs.oasis-open.org/virtio/virtio/v1.2/cs01/virtio-v1.2-cs01.html,
geraadpleegd 28 oktober 2026). De volgorde inhoud → FLUSH → tabel → FLUSH →
ACK steunt op die garantie; een emulator die haar niet nakomt valt buiten dit
bewijs.

**Testbewijs:** hosttests `services/volume/tests/` (incl. uitputtende
crashsimulatie en de test met twee gemengde schrijfpogingen),
`tests/test_volume_disk.py`, en de native gate `tools/test_volume.py` met
bewijs in `docs/verification/m6e/` en
[M6E-ACCEPTANCE](verification/M6E-ACCEPTANCE.md). Wat daar niet staat, is
niet uitgevoerd.

**Besluit:** grote documenten en ruimtehergebruik (S04/S05) krijgen een nieuw
userspace-volumeformaat met twee alternerende, per sector verzegelde
tabelkopieën en 4 KiB-inhoudschunks. Een mutatie schrijft alleen naar chunks
die de duurzame tabel niet refereert; daarna FLUSH, de volgende tabel in de
oudere slot, FLUSH, ACK. Vrijgekomen chunks worden door latere commits
hergebruikt. Er is geen journal en dus geen recordlimiet; een aparte
compactiestap is niet nodig, omdat er geen append-only log is om te
compacteren. Crashconsistente vrijgave vervult hier de rol die "compactie" in
criterium S04 beschrijft. Deze interpretatie wordt hier expliciet vastgelegd,
niet stilzwijgend toegepast.

**Security-invariant:** de kernel blijft een generieke blokbroker zonder
formaat- of domeinkennis (kernel/AGENTS.md: geen opslagpolicy in privileged
code). Domeinautoriteit komt uitsluitend van het kernelgebonden endpoint. Apps
krijgen geen blokrecht. Read-only recovery wordt afgedwongen door emulator,
virtio-onderhandeling, capability (alleen READ) en dienst. Een fout tijdens de
tabelwrite geeft `Unknown` zonder replay. Een niet-canonieke, volledig
verzegelde nieuwste kopie stopt herstel.

**Weerlegd als:** recovery na een crashpunt een andere staat oplevert dan
vóór of na de onderbroken stap; een write buiten de scope of naar een
gerefereerde chunk plaatsvindt; guest en onafhankelijk orakel verschillende
digests vinden; quota/Full pas ná I/O worden gemeld; of een read-only
recovery de image wijzigt.

**Reviewcorrectie (CodeRabbit, PR #47):** de oorspronkelijke verzegeling bond
een sector alleen aan slot, index en generatie. Twee onderbroken pogingen voor
dezelfde generatie konden daardoor samen een schijnbaar volledige kopie vormen
en een teruggedraaide mutatie laten herleven. Iedere sector draagt nu ook een
digest over de payload van alle sectoren van zijn kopie; recovery accepteert
alleen een kopie waarvan alle sectoren dezelfde, herberekende digest hebben.
Een gemengde kopie geldt als gescheurd.

**Restrisico:** één onleesbare sector in de nieuwste tabelkopie leidt tot
terugval naar de vorige generatie (verlies van de laatste commit). FNV is geen
authenticatie (B02 open). Proefgeometrie: 16 objecten, 64 KiB per document,
256 KiB inhoud.

**Migratie/rollback:** opt-in feature; geen automatische migratie van M6c-
volumes en geen gebruik van echte schijven. Terugdraaien = feature niet
selecteren of de M6e-commits gericht reverten.

**Eigenaar:** hoofdagent voor formaat, contract en integratie
(`services/volume`, `tools/volume_disk.py`, gate en bewijs); `kernel_m2` voor de
generieke blokbroker in `kernel/src/m2/m3/volume.rs`.
