# AUDIT-01 — architectuur, vertrouwensgrenzen en interfaces

**Bronmetadata:** primaire repositorybron: *Nieuw OS — “Add OS specialist
agent profiles”*, auteur/organisatie S.W.J.B.M. van Ieperen, 3 november 2026,
commit `beafdedfc188154fe7285848fbd411e651013f54`
([onveranderlijke snapshot](https://github.com/simonvanieperen/nieuw-os/tree/beafdedfc188154fe7285848fbd411e651013f54)).
Alle lokale `pad:regel`-citaten hieronder verwijzen naar die snapshot.
Externe normen, hardwaregedrag en vergelijkingen met andere OS'en zijn niet
onafhankelijk geverifieerd; behandel ze als `ONBEKEND` of als toetsvraag.

**Resultaat:** de onderzochte bron bevat een eigen `no_std`-kernel met
procesisolatie, capability-IPC en afzonderlijke native diensten. De architectuur
heeft aantoonbare ontwikkelbouwstenen, maar nog geen gesloten productketen voor
authenticatie, sleutelherstel, beveiligd internet, updates en dagelijkse desktop
op hetzelfde releaseprofiel. De belangrijkste blokkades zijn integratie- en
vertrouwensankerproblemen, niet het ontbreken van een eigen kernel.

## 1. Opdracht, bron en bewijsgrens

- Opdracht: “Analyseer de huidige architectuur, vertrouwensgrenzen, interfaces,
  onveranderlijke veiligheidsregels en architectuurgaten.”
- Datum: 3 november 2026, Europe/Amsterdam.
- Repository: `simonvanieperen/nieuw-os`.
- Bevroren bronbranch: `copilot/add-os-specialist-agents`.
- **Alle bronverwijzingen gelden voor commit
  `beafdedfc188154fe7285848fbd411e651013f54`.**
- Eigen beginbranch: `simonvanieperen-ideal-carnival`; begin-HEAD:
  `69809c2cb53dfb3bd77abe5352892810fba6e82d`. `git status --short` was leeg.
  Daarna conform opdracht detached uitgecheckt op de bevroren bron. De
  oorspronkelijke eigen branch is niet verplaatst. De auditwerkboom staat
  bij oplevering detached; dit is niet de bronbranch zelf.
- Rolprofiel `.github/agents/os-architect.agent.md` gelezen als instructietekst;
  geen technisch geladen custom agent. Root-`AGENTS.md`, `docs/AGENTS.md`,
  relevante kernel-/bootinstructies en de lokale `os-dev`-skill zijn gelezen.
  Voor het nieuwe rapportpad bestaat geen specifiekere geneste `AGENTS.md`.
- Alleen dit rapport wordt geschreven. Geen productcode, voortgang, status,
  commits, push, PR of externe dienst is gewijzigd/gebruikt.

Dit is een begrensde **AI-bronanalyse**, geen onafhankelijke menselijke audit,
exploitonderzoek, formeel bewijs of releasevalidatie. Afzonderlijke
specialistenonderzoeken worden hier niet als eigen bevinding overgenomen.
Historische bewijsbestanden worden uitsluitend gelezen binnen deze commit;
resultaten van hun oudere uitvoeringsbronnen worden niet naar deze HEAD
omgelabeld. De leesroute is risicogericht, niet uitputtend voor alle drivers,
parsers, apps, cryptografie of NAS-code.

Bewijsschaal: **E0** ontbrekend/ambitie; **E1** ontwerp; **E2**
hostprototype/simulatie; **E3** native code; **E4** native positieve test;
**E5** native negatieve test; **E6** herhaalde CI/releasegate; **E7**
gedocumenteerde hardwaretest; **E8** onafhankelijke menselijke audit/formele
evaluatie. Een hoog niveau geldt alleen voor de genoemde eigenschap en bron,
niet automatisch voor het gehele subsysteem. Hosttests zijn geen E4/E5.

## 2. Werkelijke architectuur en vertrouwensbasis

De concrete bronketen is:

```text
UEFI/OVMF
  -> eigen loader: ELF-validatie, geheugenoverdracht, optioneel signed-boot
  -> BootInfo / eigen paging, exceptions en scheduler
  -> procesgebonden capability-engine / begrensde int80-syscalls
  -> ring-3-init en diensten voor keys, volume, netwerk en presentatie
  -> SDK, DocumentSession en afzonderlijke native apps
```

`README.md:12-30`, `kernel/src/main.rs:1-43,65-79,105-113` en
`kernel/Cargo.toml:12-87` onderbouwen deze keten. Dit is geen Linux-/BSD-/XNU-/
seL4-kernel als onderlaag. De naam microkernel mag echter niet suggereren dat
alle apparaatcode buiten privilege draait: de ruwe NIC-broker beheert in de
kernel gepinde buffers en het apparaat (`kernel/src/m2/m3/network.rs:1-8,72-100`).
De netwerkprotocol- en bestemmingspolicy staat wel in userspace.

De TCB, de verzameling componenten waarop een garantie vertrouwt, verschilt
per eigenschap:

| Grens | Autoriteit en controle in de huidige bron | Wat de grens niet bewijst |
|---|---|---|
| Firmware/loader → kernel | Loader levert mappings en geldig BootInfo; kernel kopieert en valideert header/geheugenkaart (`kernel/src/main.rs:29-79`) | Bescherming tegen kwaadaardige firmware of een vervangen unsigned loader |
| App → kernel | Eigen PID, paginarechten/eigenaar, exacte berichtlengte en capability vóór operatie (`kernel/src/m2/m3/m4.rs:185-206,245-283`) | Rechten op gebruikersdocumenten uit een alleen numerieke account-ID |
| App → dienst | Kernel vult echte `sender_pid`; dienst moet eigen domein/epoch/operatie autoriseren (`crates/capability-core/src/lib.rs:1384-1429`; `services/network-service/src/lib.rs:330-369`) | Een authentieke afzender is niet automatisch inhoudelijk bevoegd |
| Init → diensten | Vast manifest verdeelt afzonderlijke key-admin-, app-, entropy- en blokrechten (`kernel/src/m2/m3/system.rs:205-240`) | Init is geen onschadelijke UI-component: key-admin-export is zeer gevoelige autoriteit |
| App → presentatie | Alleen compositor krijgt DisplaySource; apps eigen surface/inputkanaal; focus in userspace (`docs/DECISIONS.md:1488-1513`) | Echte account-lock, veilige wachtwoordinvoer of isolatie tussen gebruikers |
| Netwerkdienst → apparaat | PID/handle/objectgeneratie, gevalideerde kopieën, reset en quarantaine (`kernel/src/m2/m3/network.rs:225-246,313-355`) | DMA-isolatie tegen een kwaadaardig fysiek apparaat |
| Opslag/keyring → offline schijf | Native record- en key-stateproef bestaat; apart admin-ingress (`docs/implementation/B02-SECURE-RECORDS.md:287-330`) | Productgeheim, vertrouwde koude generatievloer of complete volumeversleuteling |
| Externe peer/provider → lokale data | E1-contract: identiteit, expliciete scope, ciphertext vóór upload, geen fallback (`docs/LOCAL-PRIVACY-CONTRACT.md:9-19,37-61,67-80`) | Native TLS/NAS/providerketen op het dagelijkse desktopimage |
| Gebruiker → sessie/keyvrijgave | UI-model scheidt correlatie, brokeractivatie en VmGuest (`ui/native/src/session.rs:145-174,368-405`) | De UI-toestand kan zelf geen kernelgrants intrekken (`ui/native/src/session.rs:416-421`) |

De lokale OSDev-leerbron is aanvullend en niet normatief: *Microkernel*,
OSDev-community, revisie 29930 (25 januari 2026), W001411,
https://wiki.osdev.org/index.php?title=Microkernel&oldid=29930.
Zij illustreert alleen de ontwerples dat userspacediensten herstel niet gratis
maken (lokale snapshot, bronregels 4-12). Dit is geen normatief
hardwarebewijs of veiligheidsbewijs voor Nieuw OS.

## 3. Interfaces, toestand en fouten

| Interface | Vastgelegde/aanwezige vorm | Integratieverplichting |
|---|---|---|
| BootInfo | M1-overdracht, ABI 1.1 en vaste reserveringen; latere grafische variant (`docs/implementation/BOOT-AND-TOOLCHAIN.md:5-7`; `kernel/src/main.rs:39-47`) | Firmwareprofiel, ownership en lifetime vóór reclaim; grafiek en signed-boot samen expliciet ontwerpen |
| Syscall-ABI | `int 0x80`, versie 1.x; featureprioriteit kiest minor 0–11 (`kernel/src/m2/m3.rs:1601-1611,1726-1751`) | Gemeenschappelijke ABI-validatie is geen detectie van iedere optionele voorziening |
| Message320 | 320 wirebytes, 256 payloadbytes, maximaal twee capdescriptors; expliciete codec, nulstaart/reserved (`crates/capability-core/src/lib.rs:158-229`) | Geen onbetrouwbare Rust-structcast; encode/decode en rechteigendom gescheiden houden |
| Capabilities | Lokale handles, generatie, type en rechten; subsetdelegatie, revocation en quota (`crates/capability-core/src/lib.rs:6-73,1302-1325,1367-1435`) | Geen autoriteit ontlenen aan het raden/kopiëren van andermans getal; geen impliciete administratieve grants |
| Procesbeheer | Launch-capability kiest ingebouwd image; hoogstens twee expliciet verminderde capgrants bij spawn (`kernel/src/m2/m3/supervisor.rs:286-337`) | Nog geen algemene pakket-naar-procesketen; serviceherstel mag app- en sleutelautoriteit niet wijzigen |
| Blok/volume/keyring | Gescheiden READ/WRITE/FLUSH en entropy; Message320, volume-v6, KEY1 (`docs/implementation/NATIVE-SYSTEM-INTEGRATION.md:28-33`) | Duurzame sleutelallocatie/intrekking vóór ciphertextcommit; Unknown niet als ACK behandelen |
| Wachten | Maximaal 16 gekopieerde handle/interestparen, deadlinehorizon 1000 ticks; elke herbeoordeling valideert PID en rechten (`kernel/src/m2/m3/wait.rs:5-17,31-94`) | Readiness is een hint, reserveert geen bericht; geen pointer/CopyPlan over contextwisseling bewaren |
| Netwerk-RPC | Twee vaste clients, PID-binding, epoch en bestemmingsbereik (`services/network-service/src/lib.rs:14-29,330-369`) | Van vast manifest naar intrekbare app-/peergrants zonder helperomweg |
| Sessiebroker | `begin` en `activate` met correlatie en opaque ticket; standaard Unavailable (`ui/native/src/session.rs:145-174`) | Authentieke dienst, gebruikerssleutels, atomische activatie en echte lock/revoke nog aansluiten |

**Toestands- en foutmodel.** De kernel zet een beëindigend proces op `Dying`
en voert cleanup uit; IPC-wait kan de overblijvende processen parkeren
(`kernel/src/m2/m3.rs:1800-1809`). IPC-send valideert vóór de commitgrens en
stempelt daarna de echte afzender; receive valideert de hele uitvoer vóór
queue-/slotmutatie (`crates/capability-core/src/lib.rs:1384-1439`;
`kernel/src/m2/m3/m4.rs:281-308`). Een mislukte M4-syscall moet de volledige
autoriteits-/queuestaat behouden; die assertion is aanwezig, geen hier nieuw
uitgevoerde proef (`kernel/src/m2/m3/m4.rs:317-327`).

Presentatieherstel heeft retirement → app-pauzebevestiging → constructie →
verse READY → rebind/ACK; oude kanalen mogen geen succes leveren
(`docs/implementation/NATIVE-WINDOW-SESSION.md:24-55`). Dit is een specifiek
herstelprotocol, geen universeel restartmechanisme voor willekeurige diensten.
Netwerkfalen gaat via offline/reset naar ACK of quarantaine; gepinde DMA wordt
niet vrijgegeven alleen omdat de eigenaar sterft
(`kernel/src/m2/m3/network.rs:313-355`).

## 4. Onveranderlijke veiligheidsregels

De volgende invarianten moeten ook bij verdere integratie blijven gelden.
Aanwezigheid van een regel bewijst niet dat alle toekomstige routes haar afdwingen.

1. **Geen impliciete autoriteit.** Installatie, handtekening, netwerktoestemming,
   privacyuitleg uit of een authentieke PID geeft geen document-, key-, beheer-
   of apparaatrecht. ADR-035 en ADR-058 behouden expliciet de Security Floor
   (`docs/DECISIONS.md:137-145,773-787`).
2. **Proces- en geheugenisolatie.** Geen W+X of writable alias van executable
   frames; gecontroleerde userkopieën, eigendom en lengtes. De huidige
   kopie-invariant veronderstelt één CPU/thread, geen gedeelde userframes en
   geen unmap tijdens de kopie (`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:39-57,96-100`;
   `kernel/src/m2/m3/m4.rs:185-235`).
3. **Atomiciteit en begrenzing.** Eerst alle rechten, buffers en quota
   controleren, dan commit; fouten laten geen gedeeltelijke rechtenoverdracht
   achter (`crates/capability-core/src/lib.rs:1384-1435`). EAGAIN/Quota zijn geen
   toestemming voor onbeperkt herhalen of een ruimere fallback.
4. **Intrekking stopt toekomstig gebruik.** Ook queued delegatie, oude handles,
   oude dienstepochs en helperroutes toetsen; reeds gelezen bytes zijn niet
   terug te nemen (`docs/implementation/KERNEL-AND-SECURITY-CONTRACT.md:122-155`;
   `docs/LOCAL-PRIVACY-CONTRACT.md:108`).
5. **Sleutel- en rolisolatie.** Key-admin-ingress is geen gewone clientfunctie.
   Een export onder een zelfgekozen geheim kan de hele keynamespace ontsluiten
   (`docs/implementation/B02-SECURE-RECORDS.md:280-285`). NAS-admin-/providerrechten
   mogen nooit inhoudssleutels worden.
6. **Fail closed.** Geen fallback naar unsigned boot vanuit signed-boot, geen
   succes bij onzeker I/O, geen certificaatacceptatie zonder toegestane tijdbron,
   geen lokale-AI-cloudfallback (`docs/implementation/B03-SIGNED-UPDATES.md:100-118`;
   `sdk/runtime/src/tls_time.rs:46-66`;
   `docs/LOCAL-PRIVACY-CONTRACT.md:82-91`).
7. **Bewijs blijft bron- en profielgebonden.** Oude VM-gates, een profielnaam
   en groene component-CI zijn geen geïntegreerde release, hardwaretest of
   menselijke audit (`docs/FULL-TEST-RELEASE.md:65-87,98-123`).

## 5. Bewijs- en maturiteitsmatrix

“Nu” betekent bronaanwezigheid op de bevroren commit, niet hier uitgevoerde
runtimeacceptatie. De kolom historisch beschrijft alleen ingelezen registraties.

| Eigenschap | Nu | Historisch bewijs in deze boom | Open bewijs/grens |
|---|---|---|---|
| Eigen boot/kernel en usermode | E3 | E4/E5: `docs/verification/m2-m4/ACCEPTANCE.md:3-25`, bronnen `ed09818` / `622c2ee` | Geen heruitvoering op audit-HEAD; geen fysieke bootclaim |
| Usercopy/capability-IPC/revocation | E3 | E4/E5 voor genoemde KS-09–16, inclusief gescheiden hostquota-tests (`m2-m4/ACCEPTANCE.md:18-25`) | Geen SMP/shared-memorybewijs; globale quota niet allemaal native getest |
| Gezamenlijke init/keyring/volume | E3 | E4/E5 geregistreerd in `full-release-b02-review-followup/system-gate.json:1-14`, source-v3 `532ed7a0…` | Geen complete desktop/net/updateketen of nieuwe gate hier |
| Native vensters, twee apps en herstel | E3 | E4/E5/E6 historisch: `gui-idle-20261102/INTEGRATION.md:17-47,49-71`; CI-head `2a7b7442…`, CI-merge `a50216ef…` | TCG/WHPX is VM, geen E7; geen accountauthenticatie |
| Netwerk per vaste client | E3 | ADR-044 registreert native ARP/UDP/DNS op `3879c32` (`docs/DECISIONS.md:1348-1365`) | Algemene app-/bestemmingspolicy en native TLS blijven afzonderlijk |
| TLS-random/tijdadapters | E2 hosttestbare no_std-voorbereiding; geen native TLS-dienst | Geen complete native handshake in het gelezen bewijs | E0 voor bruikbare vertrouwde CertificateTime-provider; `services/net-tls/src/lib.rs:1-9` |
| Geauthenticeerde records/key-state | E3 | Native positieve/negatieve koude deelproef beschreven in `B02-SECURE-RECORDS.md:287-330` | Productgeheim, koude antirollback, write-ahead en multiwriter niet bewezen |
| Gebruikerssessie | E1 echte account-/sleutelgrens; E3 UI-model | Geen echte accountauthenticatie aangetoond door deze module | UnavailableBroker en expliciete VmGuest zijn geen loginbewijs |
| NAS/provider en lokale AI | E1 productgrenzen; hostprototypes afzonderlijk | Privacycontract noemt E2 host-N01/N02 en simulaties (`LOCAL-PRIVACY-CONTRACT.md:4`) | Die historische statusregel is niet actueel genoeg om native afwezigheid te bewijzen; FR-P01–P06/L01–L03 blijven open (`FULL-TEST-RELEASE.md:210-218`) |
| Hardware/DMA en externe deskundigen | E1 gewenste grens | Geen E7/E8 gevalideerd in deze audit | Fysieke profielen/toestemming en menselijke review expliciet open (`FULL-TEST-RELEASE.md:81-87,225-232`) |

Een workflowbestand is slechts een gate-definitie. Bijvoorbeeld
`.github/workflows/native-system.yml:49-60,73-113` definieert meerdere aparte
proeven; dit rapport heeft geen daarvan uitgevoerd en geen live CI opgevraagd.
Het historische E6-oordeel bij GUI-idle is beperkt tot de opgeslagen bron- en
runregistraties, niet een bevestiging van huidige groene CI.

## 6. Geprioriteerde bevindingen

Prioriteiten betreffen de genoemde claim of vervolgstap. **BLOKKEREND** betekent
dat die acceptatie niet afgetekend mag worden; het blokkeert geen veilig
vervolgonderzoek. Er is hier geen concreet geëxploiteerde KRITIEK-bug vastgesteld.

### A01 — BLOKKEREND: productkeyherstel heeft nog geen productieanker

**Bewijs:** `sdk/examples/system/src/init.rs:31-35,202-225` gebruikt bewust een
publiek synthetisch acceptatiegeheim en herstelt koud met generatievloer 0.
`docs/implementation/B02-SECURE-RECORDS.md:326-330` noemt de ontbrekende vertrouwde
vloer en write-ahead expliciet. Dit is testfixturedata, geen aangetroffen
productiegeheimlek.

**Gevolg:** een correcte koude decryptieproef bewijst geen gebruikergebonden
keyvrijgave of detectie van teruggerolde sleutelstatus. Een actuele intrekking
kan pas een koude garantie krijgen met een onafhankelijk vertrouwd
versheidsanker. Het aparte admin-ingress is een goede grens, maar maakt het
herstelgeheim niet vanzelf betrouwbaar.

**Vervolg:** key-/opslag-/identiteitseigenaar moeten één persistent
allocatie-/intrekkingsprotocol afspreken, inclusief fout/Unknown, sleutelverlies,
generatievloer en herstel. Geen FR-O05/P05/B01-productaftekening op basis van deze
fixture. Zekerheid: hoog voor de bronvaststelling; productoplossing nog open.

### A02 — BLOKKEREND: sessie-UI is geen account- of lockautoriteit

**Bewijs:** `ui/native/src/session.rs:157-174,368-421` heeft een weigerende
standaardbroker, een afzonderlijke VmGuest-route en een `lock()` die alleen
modelstaat wist. Het contract benoemt dit eerlijk
(`docs/implementation/SESSION-AND-DESKTOP.md:98-142`).

**Gevolg:** een zichtbaar aanmeld-/slotscherm bewijst geen echte wachtwoordcheck,
per-gebruikerkeyvrijgave, inputafscherming of intrekking van appcaps.
`PROFILES-AND-USERS.md:154-183` vereist juist al deze grenzen. Dit is een
ontbrekende productketen, geen aangetoonde authenticatiebypass van de huidige
UnavailableBroker.

**Vervolg:** authentieke sessiedienst plus keyservice en compositor atomisch
verbinden; test verkeerde/stale tickets, lock tijdens RPC, gebruikerswissel,
achtergebleven klembord/surfaces en afgebroken keyvrijgave. FR-S01/S02 open houden.
Zekerheid: hoog.

### A03 — HOOG: desktop en geauthenticeerde updateboot zijn gescheiden paden

**Bewijs:** `tools/build.py:115-139` kiest voor vensters `desktop-mode` en voor
updateprofielen afzonderlijk `signed-boot`; de Windows-pakketvalidator verwacht
alleen de grafische loader (`tools/package_windows_qemu.py:111-125`).
`boot/loader/Cargo.toml:7-15` verbindt desktop-mode niet met signed-boot.
Zonder die feature leest de loader direct de ESP-kernel
(`boot/loader/src/main.rs:518-531`).

**Gevolg:** native desktopbewijs en signed-updatebewijs mogen niet opgeteld
worden tot een geauthenticeerd dagelijks productimage. Het huidige unsigned
VM-pad is niet op zichzelf een schending van het testprofiel, maar is geen
Security-Floor-productboot. Ondertekening van de kernel bewijst bovendien niet
de authenticiteit van firmware/loader zelf.

**Vervolg:** één grafisch signed-imagecontract, trust root en herstel-/datamigratie
vastleggen; pakketvalidator, bouwer en negatieve boot-/updateproeven samen
aansluiten. Behoud de oude profielen als regressie, geen stille unsigned fallback.
Zekerheid: hoog.

### A04 — HOOG: native beveiligd transport stopt nog vóór de volledige TLS-keten

**Bewijs:** `services/net-tls/src/lib.rs:1-9` en
`services/net-tls/README.md:8-11,36-45` leveren randvoorwaarden, geen
cryptoprovider/handshake/dienst. `sdk/runtime/src/tls_time.rs:29-66` laat geen
huidige klok toe als certificaattijd. Dit is bewust gesloten falen, geen bug die
met RTC als fallback mag worden “opgelost”.

**Gevolg:** DNS/TCP en een randomadapter leveren samen nog geen geauthenticeerde
internet-/NAS-peer. Ook de backend-KEM/signer-RNG en identity-/certificaatpolicy
moeten aangesloten worden. FR-N03 en daarop bouwende NAS-pairing blijven open.

**Vervolg:** onderbouwd tijd-/freshnesscontract, onderhouden provider en
hybride/authenticatiepolicy, daarna één native positieve en negatieve
transportketen. Zekerheid: hoog.

### A05 — HOOG: VM-FR-resourceprofiel volgt niet uit meer QEMU-RAM

**Bewijs:** `kernel/src/m2.rs:13-17` gebruikt 65.536 frames, dus 256 MiB
boekhoudbereik. Hogere RAM wordt behouden, niet automatisch inzetbaar.
`kernel/src/m2/m3.rs:318` en capability-core hanteren 16 processen.
Daartegenover vraagt `docs/FULL-TEST-RELEASE.md:65-79` een gepland
8 GiB-profiel met expliciete productlimieten.

**Gevolg:** de VM met 8 GiB starten maakt de allocator en appbudgetten niet
productgeschikt. Dit bewijst niet dat een 8 GiB-VM niet kan booten; het
beschikbare allocatiebudget en de workloadacceptatie zijn het probleem.

**Vervolg:** eerst workload-/capaciteitsregister en fallibele allocatiegrenzen,
dan allocator/paging/heap en quotas per profiel uitbreiden met behoud van
nulmaken, ownership, W^X en OOM-isolatie. Zekerheid: hoog.

### A06 — HOOG: DMA- en single-coreaannames zijn harde portgrenzen

**Bewijs:** netwerkbroker houdt kernelbuffers gepind en quarantaineert bij
resetfalen (`kernel/src/m2/m3/network.rs:72-85,329-355`).
`drivers/virtio-net/src/lib.rs:90` wijst ACCESS_PLATFORM naar een ander
DMA/IOMMU-profiel. Kernel-Engine en snapshotcode vertrouwen op één CPU/IF=0
(`kernel/src/m2/m3/m4.rs:121-150`); de IPC-kopie op geen gelijktijdige
mappingwijziging (`m4.rs:223-235`).

**Gevolg:** deze software-eigenaarschapregels zijn geen hardware-DMA-firewall
of SMP-synchronisatiemodel. Een fysieke driverport of extra CPU kan bestaande
veiligheidsaannames ongeldig maken zonder syscallwijziging.

**Vervolg:** vóór zulke ports een expliciet IOMMU-/buffer-/interruptmodel en
TLB-/lock-/usercopyprotocol; test reset-zonder-ACK, laat DMA, requesterexit en
concurrente invalidatie. Geen fysieke test uitgevoerd/geautoriseerd. Zekerheid:
hoog voor begrenzing; werkelijke fysieke impact onbekend.

### A07 — MIDDEL: ABI-minor encodeert profielkeuze, geen complete featureset

**Bewijs:** de prioriteitsketen in `kernel/src/m2/m3.rs:1726-1751` rapporteert
één minor, terwijl `kernel/Cargo.toml:16-83` meerdere orthogonale mechanismen
combineert. SDK `validate_abi` controleert alleen de gemeenschappelijke
compatibiliteit; IPC-wait heeft daarom al een aparte allowlist
(`sdk/runtime/src/lib.rs:26-34,81-91`).

**Gevolg:** generieke consumenten mogen uit “minor >= x” niet de aanwezigheid
van iedere eerdere optionele voorziening afleiden. Dit is een groeirisico,
geen hier gereproduceerde verkeerde syscallselectie.

**Vervolg:** centraal profiel-/voorzieningenregister of expliciete
capability-neutrale featurequery, met ENOSYS voor afwezigheid en behoud van
bestaande major-/minorsemantiek. Test combinaties, onbekende versies en ontbrekende
features; een featurequery verleent nooit apparaat- of dienstrechten.
Zekerheid: hoog voor profielvorm, middel voor toekomstige consumentenimpact.

### A08 — MIDDEL: historische statuskoppen kunnen nieuwe integratie sturen

**Bewijs:** `docs/ARCHITECTURE.md:5` noemt zijn historische basis; regel 58
verbiedt nog leverancierscloud als OS-optie, terwijl regel 65 en het bindende
privacycontract optionele opaque provideropslag toestaan. De veiligheidscontractkop
zegt nog “niet geïmplementeerd” (`KERNEL-AND-SECURITY-CONTRACT.md:3`), terwijl
native code en oude KS-acceptatie aanwezig zijn. `PROFILES-AND-USERS.md:14-18`
beschrijft een oudere UI-stand; de nieuwere sessiespecificatie registreert
wel een no_std-model maar geen echte authenticatie.

**Gevolg:** documentatie kan zowel onterechte afwezigheid suggereren als een
verouderd verbod opnieuw invoeren. De README/PROJECT-STATE-leesroute helpt,
maar vervangt geen eenduidige contractstatus per interface.

**Vervolg:** hoofdintegrator moet historische koppen dateren/linken naar de
actuele eigenaar en bindende ADR, zonder oude resultaten te herschrijven.
Geen documentwijzigingen buiten dit rapport uitgevoerd. Zekerheid: hoog.

### A09 — ONBEKEND: integrale niet-functionele en externe bewijsgrens

De gelezen bron bewijst niet de p95-latentie, volledige kernelstackdiepte,
24-uursproductenbelasting, hardwarecompatibiliteit of onafhankelijke
deskundigenreview van het uiteindelijke image. Exacte snapshots dupliceren
tijdelijk 461.816 bytes staat en wissen alle berichtkopieën; het contract
noemt de kosten en beperkte stackinspectie expliciet
(`docs/implementation/EXACT-AUTHORITY-SNAPSHOTS.md:25-48`).
Daaruit volgt geen geconstateerde stackoverflow of ontoelaatbare latentie.

`docs/FULL-TEST-RELEASE.md:229-233` houdt externe review, duurproeven en
hardware-/reproduceerbaarheidsgates open. Er is hier geen E7/E8 vastgesteld;
ontbrekend bewijs is geen impliciet akkoord.

## 7. Contract- en integratieafhankelijkheden

| Eindketen | Noodzakelijke volgorde/afstemming | Afzonderlijk te beoordelen door |
|---|---|---|
| Echt aanmelden en persoonlijke opslag | Gebruikersidentiteit → eigen key-state/namespace → vertrouwde invoer → sessieactivatie → caps → surface/input/klembord | Identiteit/sessies, key/crypto, opslag, UI en kernelrechten |
| Internet/NAS/provider | Native sockets/bestemmingsgrants → RNG plus tijd/identity → TLS-provider → pairing → encrypted objects → sync/replay/revoke | Netwerk, crypto, NAS/sync en verifier |
| Veilige desktopupdate | Geauthenticeerde boot plus grafische loader → release/imagepolicy → atomische update/rollback → volume-/keymigratie → bruikbare recovery | Boot, updater, opslag, release en verifier |
| Herstel van dienstuitval | Autoriteit/epoch ongeldig → geen nieuwe submit → DMA-ACK/quarantaine → opruimen → verse dienst → expliciete rebind | Kernel/drivers, supervisor en eigenaar van iedere dienst |
| VM-FR/HW-FR | Meetbare workload → geheugen-/quotamodel → passende drivers → gecombineerde image → negatieve/duurproeven → apart hardwaremandaat | Kernel, drivers, performance en release |

Een dienst-READY bewijst geen duurzaamheid, een request-ID geen identiteit,
een epoch geen onafhankelijk koude rollbackanker en een ondertekend apppakket
geen recht op persoonlijke data. Dat zijn de belangrijkste scheidingen tussen
de hier aangetroffen interfaces.

## 8. Begrensde ontwerpopties en voorkeursroute

Dit zijn voorstellen, geen opdracht of reeds gerealiseerde architectuurwijziging.

| Optie | Veiligheid/complexiteit | Compatibiliteit, testbaarheid en migratie |
|---|---|---|
| A. Alleen losse componentprofielen uitbreiden | Kleine lokale diffs; ketenrisico's blijven bestaan | Gemakkelijke regressie, maar geen bewijs voor dagelijks gecombineerd gebruik |
| **B. Eén verticale productketen op bestaande interfaces** | Behoud kernel en least authority; expliciete eigenaar voor identiteit/key/boot; integratiecomplexiteit zichtbaar | Oudere profielen behouden; één nieuwe bron-/imagebinding en negatieve ketengate |
| C. Vooraf generiek service-/featureframework invoeren | Meer abstrahering en grotere wijziging van de TCB/ABI | Brede migratie en heracceptatie; voor deze concrete gaten onnodig groot |

**Voorkeur B:** herstel eerst identiteit/keyherstel als expliciete trust roots
en composeer daarna een grafisch signed-image met bestaande Message320, grants
en dienstprotocollen. Ontwerp featureontdekking alleen waar concrete consumenten
die nodig hebben. Geen kernelwissel, Linuxonderlaag of algemene “trusted app”.

Migratie moet oude volume-/key-schema's expliciet herkennen; een onbekende of
corrupte opslag nooit automatisch initialiseren. Een rollback is terugkeer naar
een toegestane image en compatibele data-/keyversie, niet het terugzetten van
intrekkingsgeneraties. De exacte versie- en herstelpolicy is nog te besluiten
door de betreffende eigenaren; dit rapport introduceert geen nieuw wireformaat.

### Voorgestelde acceptatiematrix, niet uitgevoerd

| Gate | Positief | Negatief en vereist waarneembaar resultaat |
|---|---|---|
| Gebruikerssessie/keyherstel | A meldt aan, bewaart en heropent na koude start | B, fout geheim, stale ticket, afgebroken activatie: geen key/caps/input; lock tijdens actief werk sluit de echte grens |
| Keypersistentie | Generate/revoke plus ciphertextcommit over crash | Crash vóór/tijdens/na durable keymutatie; oude blob of vloer: geen herleving, geen nonce-/indexhergebruik binnen het gekozen contract |
| Signed desktop/update | Dezelfde image toont apps en herstelt na geautoriseerde update | Verkeerde handtekening, ontbrekende slotstore, downgrade en onbruikbare nieuwe image: geen unsigned fallback; data behouden |
| Native TLS/NAS | Echte handshake/peer en ciphertext vóór upload | Verkeerde/verlopen identiteit, tijdbron ontbreekt, backend-RNG faalt, oude invite/replay/revoke: geen plaintext of alternatieve peer |
| Herstel en resources | Dienstherstart met appdraft en juiste nieuwe bindings | Oude epoch/READY, volle slots/frames, DMA-reset zonder ACK: geen extra recht, hergebruik van onveilige buffers of valse ACK |
| Gecombineerde productbelasting | Vooraf vastgelegde VM-FR-workload | OOM, volle disk, netuitval en druk: andere apps bruikbaar binnen vooraf vastgelegde budgetten; geen succes op alleen markers |

De bestaande tools en workflows blijven de basis; wijzigingen en acceptatie
horen bij een afzonderlijke implementatieopdracht. Aanbevolen reviewrollen zijn
hier alleen een afhankelijkheid, geen claim dat aanvullende agents zijn gestart.

## 9. Zelf uitgevoerde controles en beperkingen

Alle eigen controles waren lokaal en read-only, behalve de uitdrukkelijk
toegestane detached checkout en dit rapport. Geen build/test is als proxy
uitgevoerd. De belangrijkste exact uitgevoerde controlecommando's:

| Commando | Exit/resultaat |
|---|---|
| `git branch --show-current && git rev-parse HEAD && git status --short && git worktree list && git cat-file -t beafdedfc188154fe7285848fbd411e651013f54` | 0; beginbranch/HEAD hierboven, schone boom, object is commit; alleen worktreemetadata, geen andere checkout gelezen |
| `git switch --detach beafdedfc188154fe7285848fbd411e651013f54 && git rev-parse HEAD && git status --short && cat AGENTS.md && cat docs/AGENTS.md && cat .github/agents/os-architect.agent.md && find docs/audits .github .agents -name AGENTS.md -print` | 1 door nog niet bestaande `docs/audits`; checkout en voorafgaande reads geslaagd, geen vuile bron |
| `git ls-files 'docs/**/AGENTS.md' '.github/**/AGENTS.md' '.agents/**/AGENTS.md'` | 0; geen specifiekere tracked instructies voor rapport/rolpad |
| `git rev-parse --verify beafdedfc188154fe7285848fbd411e651013f54^{commit}` | 0; juiste commit aanwezig |
| `git symbolic-ref -q HEAD` | 1, verwacht voor detached HEAD; geen controlefalen |
| `git diff --exit-code beafdedfc188154fe7285848fbd411e651013f54 -- .` | 0 vóór rapport; bron ongewijzigd |
| `python3 -B .agents/skills/os-dev/scripts/zoeken.py --id W001411 --start-line 1 --max-lines 48` | 0; lokale leerbron gelezen, geen installatie/netwerk |
| `git status --short && git diff --check` | 0 vóór rapport; leeg, geen whitespacefouten in tracked bron |
| `git diff --no-index --check /dev/null docs/audits/subagents/01-architecture.md` | 1 bij beide controles na schrijven; geen whitespace-diagnostiek, wel bestandsverschil met `/dev/null`; niet als exit-0-test opgevoerd |
| `git status --short --untracked-files=all` | 0 na schrijven; uitsluitend `?? docs/audits/subagents/01-architecture.md` |
| `wc -l docs/audits/subagents/01-architecture.md` | 0; toen 440 regels, vóór de toevoeging van deze eindcontrolevermelding |
| `awk '/[ \t]+$/ { print "trailing whitespace at line " NR; bad=1 } END { exit bad }' docs/audits/subagents/01-architecture.md` | 0; geen trailing whitespace |

De volgende aanvullende read-only citaatcontrole is exact uitgevoerd, exit 0:
65 volledige bronpadverwijzingen gecontroleerd, nul ontbrekende bestanden of
ongeldige regelnummers. Dit controleert vindbaarheid, niet de juistheid van
iedere inhoudelijke gevolgtrekking.

```sh
python3 -B - <<'PY'
import pathlib, re
report = pathlib.Path('docs/audits/subagents/01-architecture.md')
pattern = r'((?:docs|kernel|crates|services|sdk|boot|ui|tools|drivers|\.github)/[A-Za-z0-9_./-]+\.(?:md|rs|py|yml|json)):([0-9]+(?:-[0-9]+)?(?:,[0-9]+(?:-[0-9]+)?)*)'
checked = 0
errors = []
for match in re.finditer(pattern, report.read_text()):
    path, ranges = match.groups()
    source = pathlib.Path(path)
    if not source.is_file():
        errors.append(f'missing: {path}')
        continue
    length = len(source.read_text().splitlines())
    for item in ranges.split(','):
        bounds = [int(n) for n in item.split('-')]
        if min(bounds) < 1 or max(bounds) > length or bounds[0] > bounds[-1]:
            errors.append(f'invalid range: {path}:{item}, lines={length}')
    checked += 1
print(f'Full-path source references checked: {checked}; errors: {len(errors)}')
for error in errors:
    print(error)
raise SystemExit(bool(errors))
PY
```

Daarnaast zijn `cat`, `git ls-files`, `rg -n`, `nl -ba … | head -N` en
`nl -ba … | sed -n 'begin,eindp'` gebruikt voor de hierboven geciteerde
bronpassages en workflow-/bewijsinventarisatie. Dat zijn leeshandelingen,
geen uitgevoerde acceptatiecommando's. De shellcalls met die leesgroepen
eindigden op 0, behalve de reeds genoemde ontbrekende rapportdirectory.
Eén verkennende `rg` noemde het niet bestaande
`tools/build_window_session.py`: rg meldde expliciet “No such file or directory”;
de pipeline naar `head` eindigde op 0. Het bestaan of slagen van dat script
wordt niet geclaimd; werkelijke build-/pakketcode is daarna rechtstreeks gelezen.
Sommige brede toolweergaven waren afgekapt; beslissende passages zijn vervolgens
in kleinere bronranges gelezen. Niet-zichtbare inhoud telt niet als bewijs.

**Niet uitgevoerd:** Cargo/Clippy/build, preflight, QEMU/native gates, live
GitHub-/PR-/CI-controle, netwerk-/NAS-/TLS-proeven, integrale hashherberekening
van alle historische artifacts, benchmarks, fuzzing, hardwaretests,
gebruikersproeven, formele verificatie en menselijke audit. Geen externe
CodeRabbit-uitvoering wegens de expliciete lokale/externe-diensten-grens.
Geen gegenereerde binaries, caches of acceptatielogs geproduceerd.

**Aannames en resterend risico:** de vertrouwde ontwikkelhost en Gitobjecten
worden als leesbasis aangenomen; artifactauthenticiteit is hier niet onafhankelijk
geattesteerd. Alleen de benoemde grensroutes zijn geïnspecteerd. Geen uitspraak
over afwezigheid van fouten elders of algemene veiligheid/productierijpheid/
compatibiliteit. De zichtbare bestaande score is 41/100
(`README.md:35-46`); voorstel **41 → 41**, geen nieuw acceptatiebewijs en geen
score-/roadmapbestand gewijzigd. De hoofdagent beheert die herbeoordeling.

## 10. Overdracht

Rapport: `docs/audits/subagents/01-architecture.md`, bron
`beafdedfc188154fe7285848fbd411e651013f54`, eigen werkboom detached.
Geen uitvoeringsblocker voor deze audit; productblokkades A01/A02 en de
integratiegaten A03–A06 blijven open. De hoofdagent kan dit ene bestand overnemen
en met de overige rapporten integreren. Geen commit/push/PR aangemaakt.
