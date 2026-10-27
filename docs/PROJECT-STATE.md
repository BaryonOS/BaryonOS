# Actuele projectstatus

Bijgewerkt: 27 oktober 2026. Eerste bouwbatch B00/B01/P02/Q01.
Repository: https://github.com/simonvanieperen/nieuw-os (privé).
Werkbranch: `codex/first-build`; basiscommit `04bb3a388efd6b873177b6b3bc650f9ebfd19ba6`.
De commit die dit bestand bevat is de voortgangssnapshot; `git log -1` geeft zijn exacte ID.

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
| `node --test ui/prototype/tests/state.test.mjs` | exit0; 22 tests | uitvoer in chat; definitief browserrapport volgt |
| `node --check ui/prototype/app.js` | exit0 | uitgevoerd na laatste toetsenbordcorrectie |

De bootreeks omvat drie koude starts (ruwe QEMU-exit33), ontbrekende/corrupte kernel (35), echt stale-key EBS-herstel (33), kernelpanic (35) en een echte guest-timeout van60s. Normale artifacts zijn daarna hersteld. De ruwe lokale bestanden blijven in `build/run/` en `build/logs/`; controleerbare tekst-/hashrapporten zijn meegecommit.

Browserflows zijn daadwerkelijk doorlopen: onboarding, toegang weigeren/toestaan/intrekken, Ctrl+S, herstellen als nieuwe versie, NAS-koppeling en fouttoestanden, assistentuitkomst/model-/geheugenfouten, geldige/ongeldige voorbeeldinstallatie. Schermen op1440×900,1024×768,390×844 en320px hebben geen horizontale overflow. **Laatste modal-Tab-correctie wacht nog op runtimehercontrole**; daarom is P02 nog geen volledig afgeronde toegankelijkheidspoort. Werkelijke200%-browserzoom en schermlezer-/gebruikersproeven zijn nog niet aangetoond.

## Besluiten en herstel

De oorspronkelijke vaste kernelspan2–18MiB overlapt de werkelijke UEFI-kaart. Nu32–48MiB, dezelfde16MiB-omvang, en BootInfo1.1 met ongewijzigde128byte-layout. De read-only ESP gebruikt virtio omdat de voorgeschreven IDE-drive werd geweigerd. Een statisch opgeloste GOT is expliciet in .data geplaatst; runtime-relocaties blijven verboden. Zie DECISIONS.md en DEVELOPMENT.md.

## Niet geïmplementeerd of bewezen

Gebruikersprocessen, eigen paginatabellen/W^X-afdwinging, capabilities/IPC in de kernel, drivers, Secure Boot, PQC, fysieke hardwarecompatibiliteit, echte NAS/AI-diensten, native UI en QPU-besturing. De UI-rechten zijn mockgedrag. Geen productie- of veiligheidscertificering en geen absolute quantum-/veiligheidsclaims.

## Eerstvolgende stap en overdracht

1. Runtimehercontrole van modal Tab/Shift+Tab en definitief browser-/netwerkrapport vastleggen.
2. Eerste bouwbatch als concept-PR aanbieden; bron, lockfiles en bewijs zijn op de werkbranch te hervatten.
3. Daarna M2: eigen exceptionafhandeling en paginatabellen met negatieve geheugenbeschermingstests, vóór gebruikersprocessen.

Gebruik DEVELOPMENT.md voor de exacte startcommando's. Werk voortaan na iedere afgebakende stap en vóór langdurige controles deze status bij, commit de bron/bewijs/open punten en push de werkbranch. Een lokale wijziging of een alleen lokale commit is nog niet op GitHub opgeslagen. Wijzigingen die nog niet alle controles hebben doorstaan worden expliciet als tussenstand gemarkeerd.
