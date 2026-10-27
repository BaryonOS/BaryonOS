# Actuele projectstatus

## Actieve ontwikkelfase M2–M4

27 oktober 2026: actief op `codex/m2-m4-isolation`, gestart vanaf `144d952`. Eindcriteria en eigenaren staan in [WORK-QUEUE.md](WORK-QUEUE.md). De samengestelde M2–M3-poort is geslaagd op `6802b17`; M4 is nu expliciet vrijgegeven en in implementatie. De volledige M1–M4-opdracht is nog niet afgerond.

De privébranch wordt beoordeelbaar bewaard in [concept-PR #2](https://github.com/simonvanieperen/nieuw-os/pull/2), met `codex/first-build` als basis. `main` blijft ongewijzigd. `python -u tools/test_milestones.py --through m3` eindigde met exit0: acht M2-groepen en twintig M3-groepen, samen 25 echte QEMU-runs. M2-hostlaag bevat nu 25 tests, M3 afzonderlijk twaalf coretests en dertien user-artifacttests. `capture_gate.py m2` en `m3` controleerden alle serial/exit/artifacthashes opnieuw. Bewijs: `docs/verification/m2-m4/m2/` en `m3/`; gerichte review/herstel in `m3-review.md`.

M3 levert echte private adresruimten, int80/iretq, PIT/PIC-preëmptie, twee zestien-worker-rondes en herhaalde cleanup. Vijftien afzonderlijke usercases dekken peer-read/write, kernelwrite, heap/stack-NX, guards, CLI/x87/XMM/AVX/UD, normale exit, B exit7 en ongeldige return-RSP. Een afzonderlijke boot controleert idle na beide exits; de achttiende boot is het default-M1-herstel. De idleproef ziet acht timerwakeups met stabiele resourcebalans. AVX op qemu64 veroorzaakt #UD; dit is geen bewijs van AVX-gebruik of FP-contextswitching. Gastcycli zijn diagnostiek, geen realtimegarantie.

M2: `python -u tools/test_milestones.py` exit0, acht groepen, twintig hosttests (6 bootprotocol, 7 ELF-parser, 7 framebeheer), zeven echte QEMU-runs. Eigen CR3/GDT/TSS/IDT, frame-eigenaars, nulmaken, quota en fysieke W^X-aliascontrole zijn uitgevoerd. Afzonderlijke RO-, NX- en null-pagefaults slagen; kernel-UD stopt gecontroleerd en een onbruikbare MAIN-stack wordt via echte double-fault/IST1 afgehandeld. De default-M1-regressie slaagt. R02 vond een alias-unmapfout; herstel en negatieve tests zijn meegenomen. Compact bewijs: `docs/verification/m2-m4/m2/`; ruwe runs onder `build/run/`. NMI-overname is niet geïnjecteerd; userheap/-stack en echte userisolatie volgen in M3.

Q01-contractcontrole herstelde een CDF-afrondingsfout; 25 Rust-tests en vijf fixtures slagen. Bewijs: `docs/verification/m2-m4/quantum/`. Dit blijft lokale hostsimulatie. De volgende sectie bewaart het eerdere basischeckpoint; historische beperkingen daar beschrijven die basis.

Tussencheckpoint M4: dependencyvrije `capability-core` is toegevoegd met 21 hosttransactietests, formattering, Clippy-Dwarnings en no_std-check (alle exit0); bronhashes en commandexits zijn opnieuw gecontroleerd. De onafhankelijke review liet foutvolgorde bij twee transfercaps herstellen. Bewijs: `docs/verification/m2-m4/capability-core/`. Afzonderlijke IPC-/large-userfixtures hebben respectievelijk dertien en zes host-artifacttests. Workspace, lockfile en drivers zijn gecheckpoint; de nieuwe M4-kernelbron en 2048-mappingarena zijn op dit checkpoint nog in lokale integratie. De M4-features zijn daarom nog geen geslaagd resultaat; de rootpoort en gecombineerde regressie volgen na de complete bronoverdracht.

## Basischeckpoint eerste batch

Aanvulling P02V op deze basis: skiplink behoudt route en documentdraft, openerfocus is hersteld, zoeken ondersteunt documenttype en de desktopinspector is 288px. De actuele suite heeft 23/23 geslaagde tests; browsermetingen, bronhashes en drie screenshots staan onder `ui/prototype/verification/`. Echte 200%-zoom, schermlezer- en menselijke taakproeven blijven open. Dit is uitsluitend het hostprototype.

Bijgewerkt: 27 oktober 2026. Eerste bouwbatch B00/B01/P02/Q01.
Repository: https://github.com/simonvanieperen/nieuw-os (privé).
Werkbranch: `codex/first-build`; basiscommit `04bb3a388efd6b873177b6b3bc650f9ebfd19ba6`.
Eerste broncheckpoint: `7d2a76c736f1187601428534c66cc18c724ee199` (remote bevestigd). De commit die dit bestand bevat is de actuele voortgangssnapshot; `git log -1` geeft zijn exacte ID.

## Geïmplementeerd

- B00: afzonderlijke lokale Windows-toolomgeving; Rust1.90.0/MSVC, twee cross-targets, QEMU11.1.0 en bijpassende packaged firmware. Geen globale PATH-wijziging.
- B01: eigen UEFI-loader, veilige ELF-laadplanning, BootInfo ABI1.1, assemblyoverdracht en eigen Rust-kernel. Werkelijk in QEMU gestart na ExitBootServices.
- P02: dependencyvrij Materiaal-browserprototype, zeven routes, mockbroker, tijdelijke documentrechten, versieherstel, NAS-/AI-fouttoestanden. Fictieve data; geen native OS-integratie.
- Q01: lokale Rust-statevector-simulator (max12 qubits), no_std+alloc-core en strikte JSON-CLI. Hostsimulatie; geen QPU of OS-port.

## Werkelijk uitgevoerd

| Opdracht | Resultaat | Bewijs in repository |
|---|---|---|
| `python tools/preflight.py --strict` | exit0; tools/targets/firmware gevonden | `docs/verification/toolchain-provenance.json` en DEVELOPMENT.md |
| `python tools/build.py` | exit0; PE32+ en ELF64, drie laadsegmenten | `docs/verification/boot-manifest.json` |
| `python -u tools/test.py` | exit0; 10 acceptatiegroepen, 13 hosttests | `docs/verification/boot-suite.json`, `boot-runs.json`, `boot-host-tests.txt` |
| `python experiments/quantum/verify.py` | exit0; 23 Rust-tests, fixtures, Clippy, no_std-check | `docs/verification/quantum-report.json`, `quantum-tests.txt` |
| `node --test ui/prototype/tests/state.test.mjs` | exit0; 22 tests | `docs/verification/ui-tests.txt` |
| `node --check ui/prototype/app.js` | exit0 | uitgevoerd na laatste toetsenbordcorrectie |

De bootreeks omvat drie koude starts (ruwe QEMU-exit33), ontbrekende/corrupte kernel (35), echt stale-key EBS-herstel (33), kernelpanic (35) en een echte guest-timeout van60s. Normale artifacts zijn daarna hersteld. De ruwe lokale bestanden blijven in `build/run/` en `build/logs/`; controleerbare tekst-/hashrapporten zijn meegecommit.

Browserflows zijn daadwerkelijk doorlopen: onboarding, toegang weigeren/toestaan/intrekken, Ctrl+S, herstellen als nieuwe versie, NAS-koppeling en fouttoestanden, assistentuitkomst/model-/geheugenfouten, geldige/ongeldige voorbeeldinstallatie. Schermen op1440×900,1024×768,390×844 en320px hebben geen horizontale overflow. De laatste modal-Tab-correctie is werkelijk hergecontroleerd: beginfocus Annuleer, Shift+Tab naar toestemming, Tab terug naar Annuleer, Ctrl+K behoudt de dialoog en Escape herstelt Bewerken-focus. Ook op320px blijft focus in de dialoog. Het netwerkrapport bevat zeven initiële eigen loopbackassets en nul aanvullende verzoeken tijdens de gecontroleerde flows; browserconsole zonder warnings/errors. Bewijs: `docs/verification/browser-report.json` en `screenshots/`. Na hervatten gaf poort8765 lege responses; de eindcontrole gebruikte de afzonderlijk gecontroleerde loopbackpoort8776. Werkelijke200%-browserzoom en schermlezer-/gebruikersproeven zijn nog niet aangetoond; de volledige toegankelijkheidspoort blijft daarom open.

## Besluiten en herstel

De oorspronkelijke vaste kernelspan2–18MiB overlapt de werkelijke UEFI-kaart. Nu32–48MiB, dezelfde16MiB-omvang, en BootInfo1.1 met ongewijzigde128byte-layout. De read-only ESP gebruikt virtio omdat de voorgeschreven IDE-drive werd geweigerd. Een statisch opgeloste GOT is expliciet in .data geplaatst; runtime-relocaties blijven verboden. Zie DECISIONS.md en DEVELOPMENT.md.

## Niet geïmplementeerd of bewezen

Gebruikersprocessen, eigen paginatabellen/W^X-afdwinging, capabilities/IPC in de kernel, drivers, Secure Boot, PQC, fysieke hardwarecompatibiliteit, echte NAS/AI-diensten, native UI en QPU-besturing. De UI-rechten zijn mockgedrag. Geen productie- of veiligheidscertificering en geen absolute quantum-/veiligheidsclaims.

## Eerstvolgende stap en overdracht

1. Concept-PR #1 bevat de eerste batch: https://github.com/simonvanieperen/nieuw-os/pull/1. Bron, lockfiles, bewijs en expliciete beperkingen staan op de werkbranch.
2. Resterende P02-validatie: echte200%-browserzoom en schermlezer-/gebruikersproef. Er is geen conformiteitsclaim.
3. Volgende technische implementatiestap M2: eigen exceptionafhandeling en paginatabellen met negatieve geheugenbeschermingstests, vóór gebruikersprocessen.

Gebruik DEVELOPMENT.md voor de exacte startcommando's. Werk voortaan na iedere afgebakende stap en vóór langdurige controles deze status bij, commit de bron/bewijs/open punten en push de werkbranch. Een lokale wijziging of een alleen lokale commit is nog niet op GitHub opgeslagen. Wijzigingen die nog niet alle controles hebben doorstaan worden expliciet als tussenstand gemarkeerd.
