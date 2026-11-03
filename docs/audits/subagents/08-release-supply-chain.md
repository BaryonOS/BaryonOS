# AUDIT-08 — release en softwareleveringsketen

Datum: 3 november 2026 (Europe/Amsterdam). Opdracht: **"Beoordeel builds,
dependencies, SBOM, signing, updates, rollback, sleutelbeheer en releaseherkomst."**
Gewone chatsessie; `.github/agents/os-release-engineer.agent.md:9-31` is gelezen
als rolinstructie, niet als technisch geladen customagent.

## Conclusie en bronafbakening

**Geen grond om deze bron als distributierelease af te tekenen.** Er is wel
native updatecode, met bewaard positief en negatief QEMU-bewijs, inclusief
vooraf vastgepinde sleutelrotatie. De belangrijkste releaseblokkades zijn de
openbare testvertrouwensankers, ontbrekende firmwareauthenticatie van de loader,
ontbrekende complete release-SBOM/licentiebeslissing, geen aangetoonde hermetische
reproduceerbaarheid en onvolledig productbreed herstel-/sleutelbeheer.
Dit is een begrensde bron- en bewijsbeoordeling, geen exploitonderzoek,
onafhankelijke menselijke audit, conformiteitsverklaring of productrijpheidsclaim.

| Gegeven | Waarde |
|---|---|
| Bevroren bronbranch | `copilot/add-os-specialist-agents` |
| Bevroren broncommit | `beafdedfc188154fe7285848fbd411e651013f54` |
| Eigen beginbranch | `simonvanieperen-sturdy-tribble` |
| Eigen begin-HEAD | `69809c2cb53dfb3bd77abe5352892810fba6e82d` |
| Eigen werkpad | `/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-sturdy-tribble` |
| Toestand na bronselectie | detached HEAD op exact de bevroren bron; aanvankelijk schone werkboom |
| Schrijfgrens | Uitsluitend dit rapport; geen commits, push, PR, roadmap-, score- of statuswijzigingen |
| PR-/CI-livegegevens | Niet opgehaald: externe diensten zijn voor deze opdracht uitgesloten |

De bronbranch en haar lokale `origin/`-referentie wijzen bij de lokale controle
beide naar de bevroren commit; dat is geen verse remotecontrole.
De maincheckout is niet gelezen of gewijzigd. De bestaande
`docs/PROJECT-STATE.md:3-46` beschrijft een documentintegratie en historische
tests, geen nieuwe volledige releasevalidatie van deze bron.

Gelezen kaders: root-`AGENTS.md`, `docs/AGENTS.md`, het rolprofiel,
README, actuele checkpoint, privacycontract, relevante ADR's en
`docs/FULL-TEST-RELEASE.md`, in het bijzonder regels 96-117, 226-233 en 541-565.
De lokale OS-dev-skill is gebruikt. De toolchainroute verwijst naar
OSDev: *LLVM Cross-Compiler*, W002968, revisie 27941, 9 mei 2023
(`.agents/skills/os-dev/references/corpus/wiki-07.md:5127-5135`);
die leerbron is geen releasebeveiligingsnorm. Geen externe bron is opgehaald.

## Bewijsschaal

E0 = ontbrekend of ambitie; E1 = ontwerp; E2 = hostprototype/simulatie;
E3 = native code; E4 = native positief; E5 = native negatief;
E6 = herhaalde CI-/releasegate; E7 = gedocumenteerde hardwaretest;
E8 = onafhankelijke menselijke audit/formele evaluatie.
Een niveau geldt uitsluitend voor de genoemde eigenschap en bron.
Een hostcontrole op JSON of hashes is geen nieuwe native uitvoering.

## Release- en bewijsmatrix

| Onderdeel | Huidige bron / primaire vindplaats | Aanwezig bewijs en grens |
|---|---|---|
| Eigen targets en toolchain | `rust-toolchain.toml:1-5`; `.cargo/config.toml:1-17` | Rust 1.90.0, x86_64 UEFI en bare metal; static linking en geen kernel-build-ID. Configuratie, geen bewijs van bit-identieke builds |
| Locking | `tools/build.py:321-327`; `Cargo.lock`; `sdk/Cargo.lock` | `--locked` voor loader/kernel; ontbrekend rootlock wordt eerst gegenereerd. Ontwikkelgemak, geen fail-closed release-ingang bij ontbrekend lockfile |
| Buildherkomst | `tools/build.py:328-333,379-380`; `tools/common.py:68-84` | E2 hostmanifest met bronvingerafdruk en loader-/kernelhashes. Geen ondertekende herkomstverklaring met volledige builder-/tool-/omgevingbinding |
| Omgevingsbeheersing | `tools/common.py:26-55`; `.github/workflows/signed-updates.yml:19-40` | Bestaande hostomgeving en PATH worden overgenomen; CI haalt actuele apt-pakketten op. Niet hermetisch vastgelegd |
| CI-hardening | `.github/workflows/ci.yml:9-60`; `signed-updates.yml:9-45` | Read-only token, checkout zonder persistente credentials, actions op commit-SHA, Rustversie vast. E1/E3 workflowbron; geen eigen verse CI-run |
| Native imageauthenticiteit | `crates/update-core/src/lib.rs:149-160`; `boot/loader/src/signed.rs:14-18,166-190` | E3; hybride manifestverificatie en imagehash, ELF-parse. Historisch E4/E5 voor beide algoritmen en corruptie |
| Firmwarevertrouwensbasis | `docs/implementation/B03-SIGNED-UPDATES.md:50-56`; `boot/loader/src/signed.rs:123-163` | E0 voor geauthenticeerde firmware→loader-releaseketen; B03 begint pas bij de loader. Gewone NV+BS-versievariabele, geen aangetoonde hardwarematige monotone root |
| Signing-autoriteit | `crates/update-core/src/policy.rs:19-62`; `src/test_policy.rs:1-44` | E3 voor gebonden sleutelparen, intrekking en releaseversievensters. E4/E5 historisch voor de openbare QEMU-policy; E0 voor productieceremonie en veilige beleidsdistributie |
| Update-installatie | `sdk/examples/update/src/service.rs:4-10,254-320`; `docs/implementation/B03-SIGNED-UPDATES.md:41-49` | E3; niet-draaiend slot, verificatie voor writes, image→flush→manifest→flush→header→flush. Historisch E4/E5 installatie, hervatten en negatieve inbox |
| Rollback en downgrade | `boot/loader/src/signed.rs:311-345`; `crates/update-core/src/lib.rs:385-445` | E3; terugval naar ander geldig slot boven vloer; vloer stijgt voor overdracht. Historisch E5 downgradeweigering. Geen aangetoonde gezondheidsbevestiging na succesvolle nieuwe productsessie |
| App-pakketketen | `crates/app-package-core/src/lib.rs:6-10,328-339,410-435` | Ondertekende pakket-/publisher- en trustbroncode bestaat; geen basis om pakketupdates geheel E0 te noemen. Volledige native productapp-update-/rollbackketen hier niet vastgesteld |
| SBOM en notices | `docs/implementation/EUROPE-AND-RELEASE.md:31-39`; `docs/third-party/README.md:28-32` | E1 eis; verspreide dependencyregistraties aanwezig. E0 voor complete releasegebonden machineleesbare SBOM, inclusief toolchain/firmware/fonts/modellen |
| Support en incidentrespons | `SECURITY.md:1-8` | Expliciet geen ondersteunde release, securitycontact, responstermijn of productiekeys. E0 uitvoering, E1 vereisten |
| Integrale releasegate | `docs/FULL-TEST-RELEASE.md:226-233` | FR-B02/B03 en FR-R01/R02/R03/R04 blijven open; losse B03-imageproef is geen VM-FR/HW-FR-releaseacceptatie |

### Historisch native bewijs, niet opnieuw uitgevoerd

`docs/verification/B03-ACCEPTANCE.md:23-43,60-115` beschrijft de oorspronkelijke
13 stages. De vermelding daar dat sleutelrotatie niet bewezen is, moet worden
gelezen naast de latere aanvulling:
`docs/verification/B03-KEY-POLICY-1.md:3-46,85-119,200-205`.

| Archief | Bronbinding | Eigen read-only vaststelling |
|---|---|---|
| `docs/verification/b03/update-gate.json` | fingerprint `07994d47461a1dd5497eda995f87dc896938c3e82244617c6cf3993ee73f6137` | JSON meldt completed, passed, 13 geslaagde stages |
| `b03-key-policy-main748/update-gate.json` onder dezelfde verificationmap | testcommit `933591d4f9aadca357a3e3a4f1c1fd40c8734712`; fingerprint `1e38e27ba67c7c6793931cfd59380c1c66210ad25b0a2bd02828bf69252901c1` | JSON meldt 18/18; historische herhaling |
| `b03-key-policy-main94/update-gate.json` | testcommit `33cc4a7639e5bd4d0cb9a5bb65e40618a130bd00`; fingerprint `c6b90b631f139a4dd307f65bd3ad49e0376e0b1fee63367869dc92c3f50071fa` | JSON meldt 18/18; historische herhaling |
| `b03-key-policy-mainca45/update-gate.json` | testcommit `69d5c3acce8a97e38895f284f13d3de3dbf4a5d9`; fingerprint `df5a6f39dc081340d8d40dd6c1ba1eaa7bc40ecf372e728ba1df67120b3a73b9` | JSON meldt 18/18; alle 56 bestanden uit SHA256SUMS kloppen bij eigen controle |

De drie latere reeksen bevatten `rotate-install`, `rotate-boot`, `key-revoked`,
`key-expired` en `key-not-yet-valid`, naast de 13 oorspronkelijke scenario's.
Dit rechtvaardigt historisch E4/E5 en herhaalde lokale gate-uitvoering E6 voor
die beperkte testpolicy; geen bewijs van actuele hosted-CI, productiebeleid,
hardware of menselijke audit. Eigen Gitvergelijking met testcommit `69d5c3a`
vond geen verschillen in de hieronder bij controles expliciet opgesomde
updatebestanden. Die vergelijking omvat niet alle integratieafhankelijkheden.

Hash gate-JSON:
`f1c711e78d450c828d0b854857409460fadf7b1131ba74f033025f2635318475`.
Hash SHA256SUMS:
`50e58f7da1a9f9584f63f89ade2e665cf09fdab1ca00d502f916a8415b47e022`.
Hashes beschermen de controleerbaarheid van dit archief, niet zelfstandig
de identiteit van de oorspronkelijke uitvoerder.

## Dependency- en licentiematrix

Eigen parsercontrole van alle 53 gevolgde `Cargo.lock`-bestanden vond 338 externe
entries (niet ontdubbeld), allemaal met een checksum. Dit is geen volledige
actieve dependencygraaf of vulnerabilityscan. Pathdependencies krijgen terecht
geen registrychecksum; hun lokale bron en patches moeten apart herleidbaar zijn.

| Groep | Vastlegging | Beoordeling |
|---|---|---|
| UEFI-bindings | `docs/third-party/README.md:5-26` | r-efi 7.1.0, exacte pin, checksum, MIT-route, no_std en expliciet geen buildscript gedocumenteerd |
| Updatesignatures | `crates/update-core/Cargo.toml:7-14`; `docs/DECISIONS.md:1266-1305` | ed25519-dalek 2.2.0, ml-dsa 0.1.1 en sha2 0.10.9; manifests gebruiken semverbereiken, lockfile fixeert resolutie. Geen bevinding dat deze locked build ongepinde cryptografie gebruikt |
| Keyring | `services/keyring/Cargo.toml:7-21`; `services/keyring/vendor/README.md:3-21,44-52` | Lokale HKDF/HMAC 0.13.0-varianten met archiefhashes, licenties, patch en beperkingen; onderhouds-/herbeoordelingsplicht bij updates |
| TLS | `services/net-tls/Cargo.toml:8-17` | rustls exact 0.23.46, eigen workspace/lock; rootlock is niet het hele OS-inventaris |
| Afbeeldingen | `services/image-codec/Cargo.toml:11-16`; `THIRD-PARTY.md:1-9,34-38` in dezelfde map | Exacte zune-pins; JPEG 0.5.16-rc3 lokaal gepatcht, oorspronkelijke bron/patches/notices aanwezig. Een rc-versie is geen aangetoonde kwetsbaarheid, wel een expliciet te onderhouden releasekeuze |
| Buildscripts | `kernel/build.rs:1-40`; `docs/third-party/README.md:17-22` | Eigen kernelbuildscript registreert ingesloten ELF-/versieinputs. Alle externe registrybuildscripts en proc-macros zijn zonder downloads niet integraal onderzocht |
| Hosttoolchain/firmware | `docs/verification/native-linux-toolchain/README.md:3-11` | Historische Rust-/Debian-herkomstketen met hashes en pakketverificatie; host-/ontwikkelbewijs, geen Nieuw-OS-SBOM of productiefirmwareaudit |
| Projectlicentie | `LICENSE-PLAN.md:1-13`; `EUROPE-AND-RELEASE.md:37-39` | MPL-2.0 is voorstel, niet toegepast. Verspreide correcte notices zijn geen complete distributiegoedkeuring |

Voor de vier gericht geparseerde locks: root 82 packages/56 extern;
SDK 68/50; net-tls 29/26; image-codec 6/4. Overlap maakt optellen onjuist.
Lockchecksums bewijzen package-identiteit, niet onderhoudskwaliteit,
licentiecompatibiliteit of afwezigheid van bekende kwetsbaarheden.

## Bevindingen en prioriteit

**BLOKKEREND** betekent hier aftekenblokkade voor distributie volgens de eigen
release-eisen, niet een bewezen exploiteerbare fout. Geen KRITIEK-bevinding
vastgesteld binnen deze read-only scope.

| ID | Ernst | Bevinding, bewijs en noodzakelijke aftekening |
|---|---|---|
| REL-01 | BLOKKEREND | De bestaande signed-boot-loader en dienst gebruiken openbare TEST-autoriteit, geen releaseautoriteit (`boot/loader/src/signed.rs:14-18`; `sdk/examples/update/src/service.rs:18-21`; `tools/update-keys/README.md:3-11,21-38`). Testseeds zijn bewust openbaar, geen aangetroffen productiekeylek. Vereist: afzonderlijk releaseprofiel, gecontroleerde ankerinvoer en blokkerende controle tegen testankers in distributieartefacten |
| REL-02 | BLOKKEREND | Firmware verifieert de loader in B03 niet; andere loaderprofielen behouden unsigned ESP-boot (`B03-SIGNED-UPDATES.md:50-56,100-107`; `tools/build.py:370-378`). Vereist: afgebakende firmware→loader-trustroot en afdwinging op exact het releaseprofiel. Geen claim dat Secure Boot al werkt |
| REL-03 | BLOKKEREND | Complete machineleesbare release-SBOM en gekozen projectlicentie ontbreken als aantoonbaar opleverartifact (`EUROPE-AND-RELEASE.md:31-39`; `LICENSE-PLAN.md:6-11`; `docs/third-party/README.md:30`). Vereist: actieve directe/transitieve grafen, lokale patches, licenties/notices en assets/firmware/modellen per image vastleggen en blokkeren bij ontbrekende invoer |
| REL-04 | HOOG | Build is niet aantoonbaar hermetisch/reproduceerbaar: geërfde omgeving/PATH, actuele apt en rootlockgeneratie (`tools/common.py:26-55`; `signed-updates.yml:33-36`; `tools/build.py:321-333`). Hashmanifest is aanwezig maar geen onafhankelijke dubbelbuild. Vereist: bevroren inputs/builder en twee schone bouwomgevingen met verklaarde hashvergelijking |
| REL-05 | HOOG | Vloer stijgt na cryptografische/ELF-validatie maar vóór kerneluitvoering (`boot/loader/src/signed.rs:327-341`). Een correct ondertekende v2 die daarna crasht kan de v1-terugval onder vloer uitsluiten. Dit is brongegronde inferentie, niet hier gereproduceerd. Vereist: expliciet beleid voor proefboot/gezondheidsbevestiging of geautoriseerde recovery boven vloer, met negatieve native proef en gegevensbehoud; antirollback niet stil uitschakelen |
| REL-06 | HOOG | Bestaande vooraf gepinde rotatie/intrekking is geen distributie van nieuwe trustpolicy, herstel bij compromis of intrekking van oude loaderautoriteit (`docs/DECISIONS.md:272-300`; `tools/update-keys/README.md:33-38`). Vereist: eigenaar, offline root/delegatie, versie-/rollbackbescherming, rotatie-, intrekkings- en verliesceremonies; geen private releasesleutels in CI |
| REL-07 | HOOG | Historische crashproef knipt tijdens imagecopy; geen bewijs voor onderbreking in iedere kritieke commitfase, volledige migratie of finale productsessie (`B03-ACCEPTANCE.md:69-81,109-115`; `FULL-TEST-RELEASE.md:227,230-233`). Vereist: gerichte flush-/manifest-/header-/vloerfouten, koude recovery, mislukte eerste boot en gebruikersdatabehoud op geïntegreerd image |
| REL-08 | BLOKKEREND | Ondersteuning en incidentrespons ontbreken expliciet (`SECURITY.md:2-8`). Vereist vóór distributie: werkelijke verantwoordelijken, contact, supportvensters/EOL, triage, advisories en patch-/compromisroute. Geen fictieve contactgegevens toevoegen |
| REL-09 | MIDDEL | Oud B03-acceptatiedocument noemt sleutelrotatie nog niet bewezen (`B03-ACCEPTANCE.md:109-115`), terwijl de latere brongebonden aanvulling die beperkte native testpolicy wel bewijst (`B03-KEY-POLICY-1.md:3-46`). Vereist: bij integratie duidelijk naar aanvulling verwijzen; oude resultaten niet herschrijven of productiebeleid als voltooid labelen |
| REL-10 | ONBEKEND | Actuele advisorydekking, alle externe buildscriptbijwerkingen, CI-branchregels, daadwerkelijke secretisolatie van de uitvoerder en release-attestatie zijn niet met externe/livegegevens gecontroleerd. Workflowrechten/actie-SHA's zijn wel zichtbaar; daaruit volgt geen oordeel over alle infrastructuur |

### Grenzen tussen sleutelsoorten en herstel

Releasesigning, firmwarevertrouwen, gebruikersinhoudssleutels en
NAS-/providercredentials zijn verschillende vertrouwensgrenzen. Een
keyring-geheugenhygiënepatch is geen productie-signingceremonie; een
firmwareversievloer is geen bewijs van koude gegevensantirollback.
`docs/LOCAL-PRIVACY-CONTRACT.md:19,27-33,49-53` vereist afzonderlijke
downloadpaden zonder persoonlijke context/keys en een vertrouwde
OS-clientdistributie. De update-inboxproef bewijst die netwerk-/privacyketen niet.

Voor REL-05 is terugval naar een *toegestane* vorige versie nodig, niet
onbeperkt downgraden. Voor REL-07 bewijst een emulator-kill geen fysieke
power-lossbestendigheid. FR-B03 vraagt bovendien UI, migratie en gegevensbehoud;
die eisen worden niet door slotbytes alleen afgedekt. Offline herstel is een
releasevereiste (`EUROPE-AND-RELEASE.md:32`), geen hier aangetoonde complete
gebruikersroute.

## Eigen controles en reproduceerbaarheid

Alle onderstaande controles zijn lokaal en read-only, behalve de expliciet
toegestane detached-bronselectie en het schrijven van dit rapport.
Uitvoer staat in het sessietooltranscript; geen extra logbestand aan de
repository toegevoegd. Inhoudelijke reads gebeurden met `nl -ba`, `sed`,
`head`, `rg`, `git ls-files` en `git log`; dat zijn geen testuitvoeringen.

| Exact commando | Exit / resultaat |
|---|---|
| `pwd && git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; juiste eigen worktree, beginbranch/HEAD hierboven, status leeg, bronobject is commit |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54` | 0; alleen uitgevoerd na schone eigen werkboom en aanwezige commit |
| `git log -1 --format='%H %s' && git status --porcelain=v1 && git for-each-ref --format='%(refname:short) %(objectname)' refs/heads/copilot/add-os-specialist-agents refs/remotes/origin/copilot/add-os-specialist-agents` | 0; broncommit bevestigd, werkboom vóór rapport schoon, beide bronrefs gelijk |
| `git diff --stat 69d5c3acce8a97e38895f284f13d3de3dbf4a5d9 beafdedfc188154fe7285848fbd411e651013f54 -- crates/update-core boot/loader/src/signed.rs sdk/examples/update tools/test_update.py tools/run_update.py tools/update_disk.py tools/update-tool tests/test_update_harness.py` | 0; lege uitvoer: genoemde updatebestanden verschillen niet van historische testbron |
| `(cd docs/verification/b03-key-policy-mainca45 && sha256sum --check --quiet SHA256SUMS); result=$?; printf 'B03 capture checksum exit=%s\n' "$result"; exit "$result"` | 0; alle opgeslagen capturehashes kloppen |
| `wc -l docs/verification/b03-key-policy-mainca45/SHA256SUMS` | 0; 56 regels |
| `sha256sum docs/verification/b03-key-policy-mainca45/update-gate.json docs/verification/b03-key-policy-mainca45/SHA256SUMS` | 0; hashes hierboven |

Exacte brede inventariscontrole (uitgevoerd, exit 0):

```sh
python3 -B - <<'PY'
import json, pathlib, subprocess, tomllib
files=subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
locks=[p for p in files if p.endswith('Cargo.lock')]
external=0
missing=[]
for p in locks:
 for x in tomllib.loads(pathlib.Path(p).read_text())['package']:
  if x.get('source'):
   external+=1
   if not x.get('checksum'): missing.append((p,x['name'],x['source']))
print('tracked_Cargo_locks',len(locks),'external_entries_not_deduplicated',external,'missing_checksums',missing)
print('SBOM_named_paths',[p for p in files if any(t in p.lower() for t in ['sbom','cyclonedx','.spdx'])])
for p in ['docs/verification/b03-key-policy-mainca45/update-gate.json']:
 d=json.loads(pathlib.Path(p).read_text()); print('historical_status',d['status'],'passed',d['passed'],'all_stages_passed',all(s['passed'] for s in d['stages']),'stages',len(d['stages']))
PY
```

Resultaat: 53 locks, 338 externe entries, geen ontbrekende checksums;
geen gevolgde bestandsnamen met `sbom`, `cyclonedx` of `.spdx`;
historische gate completed/passed en alle 18 stages true. De naamzoekactie
alleen bewijst geen universele afwezigheid; de SBOM-conclusie steunt tevens
op het expliciete releasecontract en de beperkte third-partyregistraties.
Een eerdere read-only Pythoncontrole parseerde aanvullend de vier hierboven
getelde locks en de vier genoemde gate-JSON's, exit 0.

Zoekmissers tijdens verkenning: `sdk/examples/update/src/main.rs` en
`tools/build_update_images.py` bestaan niet (rg meldde foutcode 2; samengestelde
shellcall eindigde 0 door latere succesvolle commands). De echte entrypoints
`sdk/examples/update/src/service.rs` en `tools/test_update.py` zijn daarna
gelezen. Ook de veronderstelde `services/keyring/vendor/hkdf-0.13.0/README.nieuw-os.md`
bestaat niet; gelezen is `services/keyring/vendor/README.md`.
Deze missers zijn geen geslaagde controles of productfouten.

Niet uitgevoerd: builds, Cargo-tests/clippy/metadata, Python-tests,
QEMU, signing/keygen, downloads/installaties, CodeRabbit/externe scans,
remote PR-/CI-inspectie, signing met echte keys, fysieke apparaten,
menselijke review, dubbelbuild of releasevalidatie. E7/E8 zijn niet aangetoond.
De bestaande testcommando's in bewijsdocumenten zijn historische uitvoering,
niet in deze audit opnieuw geslaagd.

Afsluitende documentcontroles: `git diff --no-index --check /dev/null
docs/audits/subagents/08-release-supply-chain.md` (exit 1 voor het nieuwe,
verschillende bestand; geen witruimtemeldingen, niet als exit 0 geboekt),
`git status --short` (exit 0; uitsluitend de nieuwe auditmap) en
`sha256sum docs/audits/subagents/08-release-supply-chain.md` (exit 0;
rapporthash wordt afzonderlijk aan de integrator geleverd).

## Afhankelijkheden en overdracht

Vervolg ligt bij integrator/release-eigenaar, boot/firmware-eigenaar,
update-/opslag-eigenaar en menselijke sleutel-/supportverantwoordelijken.
Eerst releaseprofiel en trustroot vastleggen; vervolgens SBOM/herkomst/licenties,
compromisherstel en recovery met antirollbackbeleid afronden en op één finale
bron testen. Pas daarna zijn schone reproduceerbaarheidsbouw, integrale
native gates en geautoriseerde fysieke/menselijke gates betekenisvol.

Aanname: bewaarde captures zijn historische uitvoer zoals gedocumenteerd;
eigen hashcontrole bewijst inhoudsbehoud, geen onafhankelijk waargenomen run.
Er zijn geen kwetsbaarheidsvrije dependencies, intacte CI-credentials of
productieveiligheid verondersteld. Geen productpunten toegekend of gewijzigd;
de hoofdagent integreert dit rapport met de andere auditsporen.
