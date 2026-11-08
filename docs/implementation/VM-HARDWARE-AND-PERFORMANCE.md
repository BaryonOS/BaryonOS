# QEMU, pc-hardware en schermvertraging

Onderzoek en afgebakende launcherwijziging, 2 november 2026. Basis:
`e08fb0e48c1bb0781e98357f588c2f73b6ae8500`. Dit document beschrijft geen
meting op mijn pc en geen fysieke-hardwareacceptatie.

## Contract en feitelijke oorzaak

Doel: de interactieve Windows-VM CPU-virtualisatie laten gebruiken wanneer
die beschikbaar is en kan initialiseren, met zichtbare diagnose. Geen
hostinstellingen wijzigen, disks/devices doorgeven of beveiligingsmaatregelen
uitschakelen. De oorspronkelijke native acceptatieprofielen blijven TCG;
Windows/WHPX-acceptatie is een afzonderlijke nog open controle.

De oorspronkelijke launcher forceerde `tcg,thread=single`, `qemu64`, één
vCPU, 256 MiB en `pc-q35-8.2`. TCG vertaalt CPU-instructies in software. Ook
`run_materiaal_preview.py` gebruikt dit expliciete profiel; zijn standaard
`timeout=45` is een afbreekgrens, geen gemeten schermtijd. Reproduceer de door
mij gemelde vertraging van circa 45 seconden per scherm met afzonderlijke
fasetijden.

Daarnaast toont de code een dure renderroute:

- `ui/surface-protocol/src/lib.rs`: tegels van 32×8 pixels, maximaal 1024
  bytes; `CHUNK=208`.
- `ui/surface-protocol/src/client.rs::Session::blit`: iedere volledige tegel
  stuurt BEGIN + vijf CHUNK + COMMIT en wacht op één bevestiging.
- `sdk/examples/compositor/src/surface.rs::Surface::step`: verwerkt maximaal
  één verzoek per aanroep. De compositor staat na iedere hoofdloop zijn
  CPU-beurt af (`yield_now`).
- `sdk/examples/materiaal/src/lib.rs::paint_step`: één tegel per app-iteratie;
  een gewijzigde view-generation begint opnieuw bij tegel nul. Er is nog
  geen protocol voor alleen gewijzigde rechthoeken.
- `kernel/src/framebuffer.rs::write_pixels`: gevalideerde scalare volatile
  stores naar een supervisor-only ongecachete framebuffer. Dit is geen
  GPU-acceleratie.

Een volledige **1280×720 oppervlakte** telt `40×90 = 3600` tegels: **25.200
verzoeken en 3600 antwoorden**, exclusief display-syscalls, input, polls en
compositorranden. Met de bestaande 16/48/16-pixelranden is de toegewezen
app-oppervlakte bij een 1280×720 scherm 1248×656: 3198 volle tegels, dus
22.386 verzoeken en 3198 antwoorden. Dit zijn exacte protocoltellingen uit
de code, geen tijdmeting. Extra virtuele cores lost deze seriële route niet op;
de kernel is bovendien nog single-core.

## Geïmplementeerd in de Windows-launcher

`Start-NieuwOS.ps1 -Acceleration Auto` (standaard):

1. Controleer bundelhashes, inclusief de nieuwe `Acceleration.ps1`-helper.
2. Vraag `-accel help` op. Een vermelde WHPX-backend bewijst alleen dat QEMU
   ermee is gebouwd.
3. Test WHPX met hetzelfde q35/CPU/geheugenprofiel, gepauzeerd via `-S`.
   Deze proef heeft geen disks, netwerk, USB, hostmappen of scherm. Een
   QMP-antwoord op `query-status` met de verwachte request-ID en status
   `paused`/`prelaunch`, capability-/quitbevestigingen en exit 0 moeten
   bevestigen dat initialisatie slaagde. De opdrachten gaan als exacte
   UTF-8-bytes met alleen LF naar QMP, zonder Windows-tekstnewlinevertaling.
   De launcher wacht eerst op de echte QMP-greeting en vervolgens op iedere
   commandobevestiging vóór hij de volgende opdracht verstuurt; handshake en
   procesexit delen één deadline van tien seconden;
   na maximaal tien seconden wordt de proef gestopt. Er draait geen gastcode.
4. Start daarna de eigen OS-VM met WHPX. Bij ontbrekende/mislukte initialisatie
   meldt Auto de oorzaak en kiest TCG. `-Acceleration Whpx` weigert fallback;
   `-Acceleration Tcg` kiest het oorspronkelijke softwareprofiel.
5. Schrijf de geselecteerde backend, reden, profiel en starttijd naar
   `state/launch.json`. Een fout tijdens de echte VM-start veroorzaakt geen
   automatische herstart of herhaling van schijfbewerkingen.

`-CheckOnly` controleert hashes en lijsten, maakt geen mutable state en start
ook geen gepauzeerde proefmachine. Het meldt daarom uitdrukkelijk dat WHPX
nog niet is geïnitialiseerd. Een geslaagde WHPX-proef bewijst geen werkende
Nieuw-OS-sessie. De launcher schakelt **geen** WHPX security-domainmitigatie
uit (`ssd=off` wordt niet gebruikt).

Het package accepteert exact de twee zevenprocesprofielen
`native-system-materiaal` en `native-system-session-preview`. Het bewaart het
werkelijke profiel in zijn manifest; de sessiepreview wordt niet als het
historische Materiaal-bewijs gepresenteerd. Beide houden dezelfde ELF-,
bron-, hash- en ESP-controles.

## Hardware gebruiken met behoud van grenzen

| Onderdeel | Eerstvolgende verantwoorde route | Bewijsgrens |
|---|---|---|
| CPU op Windows x86-64 | WHPX via Windows Hypervisor Platform en firmwarevirtualisatie | 6/6 echte helperproeven en 6/6 native Windowsruns plus Auto/WHPX/SDL-start; zie brongebonden eindrapport |
| CPU op Linux x86-64 | KVM als QEMU-backend met bestaande toegang tot `/dev/kvm` | Onderzocht; huidige native runners blijven expliciet TCG |
| CPU op Intel-Mac | HVF, mits QEMU/host dit aanbieden | Onderzocht, niet geïmplementeerd of getest |
| Apple Silicon of Windows ARM | x86-64 gast vraagt architectuuremulatie; ARM-virtualisatie versnelt deze x86-64 kernel niet rechtstreeks | Andere gastarchitectuur is afzonderlijk portwerk |
| Geheugen | Begrensd virtueel RAM; één vCPU tot SMP-invarianten zijn bewezen | Meer RAM vermindert de IPC-transacties niet |
| Beeld | Eerst renderbatches/dirty regions, daarna geïsoleerde virtio-gpu-driver | WHPX versnelt CPU-uitvoering, geen nieuwe GPU-driver |
| Toetsenbord/muis | Huidige geëmuleerde inputkanalen | Geen volledige host-USB-controller overdragen |
| YubiKey/FIDO2 | Eerst gesimuleerde protocolfouten en begrensde USB/CTAP2-service; later één expliciet gekozen testtoken | Huidige gast heeft geen werkende FIDO2-login; geen persoonlijke security key doorgeven |
| Vingerafdruklezer | Later ondersteunde driver + geïsoleerde authenticatieservice en enrollment | Host-Windows-Hello geeft geen generieke vingerafdruk-API aan deze gast |
| TPM | Software-TPM met eigen VM-state voor vroege proeven | Geen host-TPM doorgeven; de host gebruikt eigen sleutels/PCR's |
| Schijf/netwerk | Synthetische imagebestanden, read-only bundel-ESP, netwerk standaard uit | Geen fysieke hostdisk, bridge, clipboard, host-home of GPU/PCI-passthrough |

De VM draait als gewone gebruiker met actuele QEMU/firmware. CPU-virtualisatie
betekent dat gastinstructies onder controle van de hypervisor op de fysieke
CPU draaien; het geeft de gast geen vrije toegang tot hostschijven of USB.
Device-passthrough is een afzonderlijke vertrouwensgrens. Voor een latere
PCI/GPU-proef zijn IOMMU-ondersteuning, geschikte devicegroepen, driver- en
resetgedrag en een concreet geautoriseerd testapparaat nodig. Die stappen
zijn hier niet uitgevoerd. Ook TCG is geen harde sandbox: de huidige
QEMU-securitypolicy geeft de emulatie-usecase geen isolatiegaranties.

## Meting en volgende renderstap

Meet drie herhalingen per backend met dezelfde bronhash, resolutie, firmware,
schone VM-state en hostbelasting. Registreer koude boot → compositor gereed,
eerste volledig bevestigd frame, invoer → volledig nieuw frame en idle-CPU.
Gebruik monotone tijd voor intervallen; `launch.json` bevat alleen de
startconfiguratie, geen fictieve rendermetingen. Een timeout geldt als fout.
Maak TCG/WHPX-resultaten niet uitwisselbaar met bestaande native gates.

De eerstvolgende renderoptimalisatie kan zonder ruimere apprechten:
verwerk een klein begrensd aantal reeds beschikbare SUR1-berichten per
compositorbeurt en controleer klok, terminale toestand en input/control tussen
batches. Behoud sender-/capability-/epoch-/lengtecontroles en de
COMMIT-bevestiging; voorkom onbegrensd draineren/starvation. Test vastgelopen
clients, intrekking midden in een batch en controle van ieder bevestigd
frame. Dit verandert timing en vraagt native acceptatie; het is hier alleen
een voorstel. Daarna versieerbaar dirty-regiontransport; op langere termijn
een capabilitygebonden overdrachtsbuffer met expliciet ownership, begrensde
lengte en intrekking. Geef apps geen raw framebuffer of DMA-recht voor snelheid.

## Uitgevoerde controles

- PowerShell 7.5.4 op de Linux-cloudexecutor: 33 synthetische
  versnellingsbeleid-/QMP-antwoordtests (succes, ontbrekende backend,
  initfout, timeout, expliciete keuze, geen fallback, geen probe bij
  CheckOnly, ongeldige/ontbrekende/dubbele QMP-bevestiging en sequentiële
  handshake met begrensde eventverwerking) geslaagd.
- De zes launchercontroles met synthetische bundel en een fake QEMU-binary
  zijn op diezelfde Linux-PowerShell uitgevoerd; gewijzigde helperhash wordt
  vóór uitvoering geweigerd. Geen echte VM gestart door die tests.
- De echte Linux-QEMU-binary weigert de WHPX-initproef met `invalid accelerator
  whpx`; de helper rapporteert `Ready=false` en exit 1. Dat test de
  proces-/foutafhandeling, geen Windows-hypervisor.
- Package-hosttests omvatten hashbinding van de helper en behoud van het
  sessiepreviewprofiel. De integrator bewaart het finale commando/resultaat.
- Drie synthetische subprocessproeven controleren de exacte LF-only
  QMP-requestbytes en accepteren alleen bevestigde gepauzeerde initialisatie;
  `running` en een ontbrekende quitbevestiging worden geweigerd. Dit is
  Linux-procesbewijs zonder hypervisorinitialisatie, geen Windowsbewijs.
- Windows PowerShell 5.1, echte WHPX-sessie, KVM/HVF, fysieke pc/periferie en
  snelheidsvergelijking: **niet uitgevoerd**. Deze cloudresultaten beschrijven
  mijn pc niet.

Rollback: draai de launcher/helper/packagewijziging terug als afzonderlijke
commit; de expliciete `-Acceleration Tcg` behoudt ondertussen het oude
softwareprofiel. Geen hostfeature of firmware is aangepast.

## Primaire bronnen

Geraadpleegd 2 november 2026; QEMU-webdocumentatie vermeldt versie 11.1.50.
Dit is een ontwikkelhandleiding, geen bewijs dat alle opties identiek werken
in de minimaal toegelaten QEMU 10.2.1-build.

- QEMU Project. *Windows Hypervisor Platform*. Vereisten, WHPX-acceleratie,
  legacy-VGA/MMIO/SIMD-beperkingen en security-domainmitigatie.
  https://www.qemu.org/docs/master/system/whpx.html
- QEMU Project. *Introduction*. Hardwareversnellingsbackends en hosts.
  https://www.qemu.org/docs/master/system/introduction.html
- QEMU Project. *Invocation*. Backendselectie en initialisatie/fallback.
  https://www.qemu.org/docs/master/system/invocation.html
- QEMU Project. *Security*. Virtualisatie, TCG-grenzen, apparaatdoorvoer en
  gewone gebruikersrechten.
  https://www.qemu.org/docs/master/system/security.html
- QEMU Project. *QEMU TPM Device*. TPM-passthrough deelt de fysieke TPM niet
  vrij met host/gasten; software-TPM is een afzonderlijke route.
  https://www.qemu.org/docs/master/specs/tpm.html

## Lokale Windowsbevindingen van 2 november 2026

De geautoriseerde lokale Codex-sessie rapporteert voor mijn pc een Ryzen 9
9950X3D met 64 GiB en Windows 11 25H2. WHPX initialiseert zonder hostinstellingen
te wijzigen. Op de oude bundel leverde één koude proef 10,453 s met WHPX
tegenover 19,952 s met TCG tot exact hetzelfde volledige frame. Op bron
`7648a8f` slaagde de vijfstappenroute met WHPX in 43,824 s; schermovergangen
duurden 6,856–9,107 s.
Dat is uitvoeringsbewijs voor die specifieke bron, geen productlatentieclaim.
Ruwe lokale artifacts blijven bij die sessie; de gerapporteerde bron-/firmware-/
meetgrens is vastgelegd in [local-pc-reported.json](../verification/session-desktop-20261102/local-pc-reported.json).

De eerste HMP-helper gaf op Windows ten onrechte een time-out: invoer kon
vóór monitorreadiness aankomen; LF alleen verhielp dit niet. De huidige helper
wacht op de QMP-greeting en ieder gecorreleerd antwoord. De versie `51d883e`
heeft ook de platformsorteerfout in source-v3 hersteld; bytegelijke Windows-
en Linuxcheckouts leveren sindsdien rechtstreeks dezelfde sourcehash.
Geen inputbron of integriteitscontrole is verwijderd.

Testmappen horen buiten synchronisatie: OneDrive maakte bij de lokale proef
extra hardlinks, die het harnas terecht weigerde. De vervolgproef gebruikt
LocalAppData; hardlinkcontroles blijven actief. De uiteindelijke Windowsruntime
op `344dc0a` staat afzonderlijk in het eindbewijs;
de oudere resultaten worden niet als uitvoering op de nieuwe bron gepresenteerd.

De definitieve lokale reeks op `344dc0a` is afgerond: 3 WHPX- en 3 TCG-runs,
alle 8 fasen en 48 appbeelden exact, exitcodes 0. Daarna volgde een echte
Windows-start met Auto/WHPX/SDL en launcherexit 0. WHPX eerste account:
gemiddeld 6,625 s (6,481–6,823); gast → desktop: 4,070 s (3,932–4,231).
TCG: respectievelijk 13,356 s en 10,023 s. Volledige route: 31,481 s tegenover
71,982 s. Drie starts per backend op mijn pc,
niet beheerst voor achtergrondlast. Verdere bron-/pakkethashes en de
afzonderlijke idle-CPU-observatie staan in het lokale eindrapport hierboven.

## Bekende ABI 1.11 en bestaande idle-wachtroutes

2 november 2026, fixbasis `2bb84c2`. De vensterkernel biedt IPC-wait aan onder
ABI 1.11, maar de bestaande keyring- en volumedienst activeerden hun idle-wait
uitsluitend bij 1.10. Daardoor bleven deze twee diensten in dit profiel lege
queues pollen. De gedeelde, syscallvrije SDK-helper `supports_ipc_wait` herkent
nu uitsluitend de bekende 1.10- en 1.11-profielen; beide bestaande selectors
gebruiken hem. Algemene ABI-compatibiliteit, bijvoorbeeld RTC 1.9, impliceert
geen wachtvoorziening. Onbekende majors/minors blijven uitgesloten.

Keyrings RANDOM-/beheeruitsluitingen, bestaande queuevoorwaarden, absolute
deadlines, capabilityrollen, foutafhandeling en verwerking van lopend brokerwerk
zijn ongewijzigd. RANDOM- of beheer-ingress behoudt zijn bestaande pollingpad.
Er is geen wijziging aan de GUI-, SUR-/INP-protocollen of bron-hardwarepolling.

De gerichte SDK-hosttests, inclusief negatieve ABI-selectie zonder hostsyscall,
en strikte SDK-Clippy slagen. Beide echte scalar x86_64-unknown-none-diensten
bouwen en doorlopen de ELF-inspectie. Logs en servicehashes staan onder
`build/abi-wait/logs/`. De onafhankelijke diffreview vond geen blocker; geen
VM is voor deze kandidaat gestart. De uiteindelijke geïntegreerde native gates
moeten het 1.11-wachtpad nog uitvoeren.

Dit herstelt twee concrete compatibiliteitsfouten en bewijst geen volledige
VM-idle of latencywinst. De compositor, beide GUI-apps en window-init bevatten
nog eigen pollinglussen; hun verdere idle-inrichting en daadwerkelijke
idle-CPU-meting blijven open. Een CPU-belasting van 100% tijdens actief tekenen is geen
idle-meting en er worden geen U02-punten toegekend.
