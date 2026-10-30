# TASK-H03-TRANSPORT-CONTRACT-1 — virtio-snd transport-, reset- en epochnaad

Status: **contractvoorstel, niet geïmplementeerd en niet uitgevoerd.** Geen
code, IRQ, MMIO, transport, capability, syscall of gedeelde ABI in deze stap.
Rubric H03/A04; FR-H02/A06/A07 blijven open. De hostcodec in `src/` is geen
native audio. Eigenaar van dit document: TL-HW. Integratie en elke root-,
kernel-, capability- of ABI-wijziging: release/kernelmanager na aparte review
met security; dit document verleent geen schrijfrecht daarop.

## 1. Bronpinnen

| Bron | Versie / hash | Gebruikt voor |
| --- | --- | --- |
| OASIS, *Virtual I/O Device (VIRTIO) Version 1.3*, <https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html> | sha256 `3423e9507f2c363fc8e7bbdd013afc0436ed2954b79aa34db55cbf9afa07d45d`, opgehaald 30 okt 2026 | §2.4 reset (status 0, daarna geen queue-interactie), §2.7.8 used ring (geen epoch), §5.14.5 init, §5.14.6.6.5 RELEASE, §5.14.6.8 PCM I/O |
| QEMU `v10.2.1` `hw/audio/virtio-snd.c`, <https://github.com/qemu/qemu/blob/v10.2.1/hw/audio/virtio-snd.c> | sha256 `ccb7e3b6ed7937602f86bf2a3badc70e77bd4e63d2dafb06664c717c74f81114` | streamrichting, dubbele RELEASE, ongeldige TX, reset |
| QEMU `v10.2.1` `audio/wavaudio.c` | sha256 `94ad3a5456465bac6aec549bcb0c5d24790a548c84a47c9caa8954094c3eae54` | waarneemorakel voor de runtimegate (S16 ondersteund) |
| `crates/virtio-core/src/lib.rs` @ `ca45a7d` | sha256 `24dc240458ff4558eac959c0512b11e73ca9b09da28ddd5179bf2b4361cbd293` | `Policy`: epoch, `reset_completed`, quarantaine, ticket |
| `drivers/virtio-net/src/lib.rs` @ `ca45a7d` | sha256 `9d6689a2cc9c02e4ebd2f211b3d4760e95da052b168ec3c30691f99fadf8f7b5` | patroon gepinde ring/bounce, idle-used-indexweigering |
| `crates/capability-core/src/lib.rs` @ `ca45a7d` | sha256 `9c41ac47e783405901663423b0e0463d5fa665f445abfb8d62de1349debd9fb4` | manifest-only, niet-delegeerbare rechten (`NETWORK`, `DISPLAY`, `INPUT`) |
| `docs/DECISIONS.md` ADR-026 (M5) en ADR-044 (netwerkbroker) | sha256 `651fc7586d42a08fef8c545b5dfc47a3a1ae2dc2b13bee5a72b39cdfa38882d1` @ `ca45a7d` | vertrouwd kerneltransport, geen IOMMU, reset-ACK/quarantaine |
| `docs/implementation/VIRTIO-NET.md` | sha256 `2114863edb97b2dbe2310ae0b8ccf78ce371839cb6dbe7527821cb97f1787de6` | bus mastering uit + status 0 = reset bevestigd |
| `docs/verification/m5/transport-review.md` | sha256 `e16d63180001056b4521206ae2a888a2c7abe3098f2e28aae8028312e8a2f06b` | foutpaden na BME-enable, device-scope, verweesde fout |
| QEMU-detectieprobe (sessiebewijs TL-HW) | QEMU 10.2.1, OVMF 4M | `1af4:1059` op `00:08.0`, streams 2, jacks 0, chmaps 0 |

Waargenomen QEMU-gedrag (broncode, niet uitgevoerd): stream-ID's
`< streams/2 + streams%2` zijn uitvoer, de rest invoer; `streams=2` geeft dus
stream 0 = uitvoer en stream 1 = **invoer**. Stream-info biedt
`channels_min=1`, `channels_max=audiodev-kanalen`, S16 en 48 kHz. Een tweede
RELEASE van dezelfde stream roept `virtio_error` aan (device kapot), een TX met
een ongeldig of invoer-stream-ID wordt gecompleteerd met status `BAD_MSG`, hoewel §5.14.6.8 voor I/O alleen `OK` of `IO_ERR` noemt.

## 2. Vertrouwensmodel

- **TCB in dit profiel:** eigen kernel (PCI/MMIO-mapping, ringen, gepinde
  DMA-pool, usercopy), het protocolconforme QEMU-device en de audiobackend.
  Zelfde model als ADR-026/ADR-044; het transport is een **nog vertrouwd
  kerneltransport**, niet geïsoleerd.
- **Geen IOMMU.** Er is geen VT-d/AMD-Vi-ondersteuning. Begrensde descriptors en
  CPU-paging bewijzen **geen** containment van een kwaadwillig of defect device.
  Een epoch of ticket maakt DMA niet veilig; alleen een bevestigde devicefence
  (§5) beëindigt DMA op de pool. Fysieke hardware valt buiten dit contract.
- **Audiobroker (ring 3):** één geconfigureerd proces met een
  manifest-only, niet-delegeerbaar uitvoerrecht (zie open besluit B1). Geen
  PCI-, MMIO-, descriptor-, fysiek-adres- of queuekeuze. Sluiten of exit van de
  broker beëindigt de lease.
- **Apps:** geen devicecapability. Een app praat later via IPC met de broker
  (apart contract). Een geadverteerd PCM-format geeft geen appgrant; uitvoer is
  geen microfoonrecht.
- **Geen capture.** Dit contract levert geen invoerpad; de RX-queue krijgt nooit
  een buffer (§3).

## 3. PCI- en queue-eigendom

| Element | Eigenaar | Regel |
| --- | --- | --- |
| PCI-function `00:08.0`, `1af4:1059`, modern, BAR's | kernel | Vaste BDF per profiel; afwijkend ID/BDF of legacy ⇒ `Absent`. Kernel kiest features met `negotiate_features` (VERSION_1, nooit CTLS, ACCESS_PLATFORM ⇒ weigeren). |
| Config (12 bytes) | kernel, read-only | Generation-stabiele read; `Config::parse`. |
| controlq (0) | kernel | Eén uitstaand verzoek; device-readable request + device-writable response in eigen pagina. |
| eventq (1) | kernel | Vaste set lege 8-byte device-writable buffers (§5.14.5 MUST). |
| txq (2) | kernel | Alleen device-readable header+PCM en een device-writable 8-byte status (§5.14.6.8.1.2: geen device-writable PCM-buffer). |
| rxq (3) | kernel | Ring geconfigureerd maar **nooit** een buffer gepubliceerd (open besluit B5: uitschakelen in plaats van configureren). |
| Streamkeuze | kernel via codec | `first_stereo_s16_48000`; nooit `INPUT`. Voorstel VM-FR: `streams=1` (alleen uitvoer, open besluit B6). |

## 4. DMA-pool, uitvoerbuffers en levensduur (voorstel)

- Kernel-eigen, permanent gepinde, uitgelijnde RAM-pagina's (`Owner::Kernel`),
  nooit userframes en nooit vrijgegeven tot VM-stop, ook niet na mislukte init:
  één ringpagina per queue (4), één control-/eventdatapagina en twee
  TX-bouncepagina's.
- Periode: 10 ms bij 48 kHz/stereo/S16 = 1920 bytes. Vier TX-slots
  (2 × 2048 bytes per bouncepagina); header (4) en status (8) per slot in de
  control-/eventdatapagina. Maximaal 40 ms gebufferd; geen latencyclaim
  (budget is H03-werk op fysieke hardware).
- PCM komt van de broker via een volledige, eenmalige kernelcopy naar een vrij
  slot; geen gedeelde Rust-referentie naar DMA, geen app- of brokerpointer in
  een descriptor. Een ongeldige of te grote brokerbuffer wordt vóór kopiëren
  geweigerd en publiceert niets.
- Slotlevensduur: `Free → Filled → Published → Completed → Free`. Hergebruik
  alleen na een gevalideerde completion (bekend gepubliceerd descriptor-ID,
  used-len precies 8, status `OK` of `IO_ERR`; `BAD_MSG` of onbekend ⇒ fence) **of** na een bevestigde fence (§5).
  `Completed` betekent "device heeft buffer geconsumeerd" (§5.14.6.8.1.1), niet
  "hoorbaar afgespeeld".

## 5. Lease, epoch, revoke, reset en quiescence

- **Lease** = `(device-epoch, broker-PID, capability-objectsleutel)`. Elke
  syscall en elke completion-aflevering herautoriseert alle drie.
- **Epoch** is boekhouding, geen fence. De used ring heeft geen epoch (§2.7.8);
  een late completion kan daarom alleen worden uitgesloten door de fence.
- **Fence (quiescence)** = schrijf status 0, lees status 0 binnen een
  monotone deadline (§2.4.2) **én** PCI bus mastering uit en teruggelezen. Pas
  dan: slots zeroën, ring-indexen herstellen, epoch `checked_add(1)`.
  Bus mastering uit alleen is geen fence; status 0 zonder BME-off ook niet.
- **Revoke** (broker exit, cap gesloten, beheerder): weiger nieuwe submits →
  fence → open verzoeken krijgen `Revoked` met onbekende afspeelstatus; geen
  replay. RELEASE is optioneel vóór de fence en nooit vereist voor veiligheid.
- **Reset bij fout:** zelfde fence. Fence-time-out of overflow van de epoch ⇒
  `Quarantined` tot reboot; pool blijft gepind, device offline.
- **Herinit** na fence: volledige §5.14.5-reeks opnieuw, inclusief feature- en
  configcontrole; oude streamstatus wordt niet hergebruikt.

### Toestanden

`Absent`, `Offline`, `Init`, `Ready` (geen stream), `Configured`
(SET_PARAMS OK), `Prepared`, `Running`, `Stopped`, `Fencing`, `Quarantined`.
Overgangen: `Offline→Init→Ready` (§5.14.5), `Ready→Configured→Prepared→
Running⇄Stopped`, `Stopped→Ready` via één RELEASE (stream gemarkeerd als
vrijgegeven, nooit twee keer), elke toestand `→Fencing` bij fout/revoke,
`Fencing→Offline` (ACK) of `→Quarantined` (time-out).

## 6. Fouttabel

| # | Trigger | Detectie | Reactie | Negatieve test (runtime) |
| --- | --- | --- | --- | --- |
| F1 | Geen device / ander ID / legacy | PCI-enumeratie | `Absent`; broker krijgt `NoDevice`; geen DMA-pagina's uitgedeeld | QEMU zonder `virtio-sound-pci` |
| F2 | VERSION_1 ontbreekt, ACCESS_PLATFORM aangeboden | `negotiate_features` | `Offline`, geen DRIVER_OK | hostcodec (bestaand) |
| F3 | Config ongeldig, streams 0 of > 32 | `Config::parse` | `Offline` | `streams=0`-profiel |
| F4 | Geen uitvoerstream S16/48k/stereo | codec | `Ready` zonder stream; `Unsupported` | hostcodec; `streams=0` |
| F5 | Control-status ≠ OK, used-len ≠ verwacht | codec + transport | stream niet gestart; BAD_MSG/IO_ERR ⇒ fence | — |
| F6 | Control-time-out | monotone deadline | fence | geblokkeerde backend (open) |
| F7 | TX used-ID onbekend/niet gepubliceerd, used-len ≠ 8 | transport | fence; geen slot vrij | modelinjectie |
| F8 | Used-index verandert terwijl niets uitstaat (stale) | idle-check vóór publicatie | fence | modelinjectie |
| F9 | TX-status `IO_ERR`; of `BAD_MSG`/onbekend | status | `IO_ERR`: slot vrij, `IoError` aan broker, stream stoppen. `BAD_MSG`/onbekend: fence | modelinjectie (de driver stuurt zelf nooit een ongeldig stream-ID) |
| F10 | `DEVICE_NEEDS_RESET` / configwijziging | statusread | fence | dubbele RELEASE (QEMU `virtio_error`) |
| F11 | Completion na status-0-ACK | ring-snapshot na fence | §2.4.1-schending ⇒ `Quarantined` | modelinjectie |
| F12 | Fence-time-out | deadline | `Quarantined`, pool gepind | modelinjectie |
| F13 | Broker exit / cap dicht tijdens `Running` | kernel-cleanup | revoke-fence; geen replay | kill broker midden in afspelen |
| F14 | Ongeldige/te grote brokerbuffer | usercopy-validatie | `Invalid`, niets gepubliceerd | syscall met slechte pointer/lengte |
| F15 | Alle slots bezet | slottelling | `Again`, niet blokkeren | snel schrijven |
| F16 | Underrun (geen data) | broker | `Stopped`, geen fout | pauze |
| F17 | Event ongeldig (len ≠ 8, stream ≥ streams, onbekend type) | eventparser | negeren + teller; herhaald ⇒ fence | modelinjectie |
| F18 | Epoch-overflow | `checked_add` | `Quarantined` | hosttest |
| F19 | Tweede RELEASE | streamvlag | geweigerd vóór verzenden | hosttest |

## 7. Volgende minimale runtimegate (VM-SND-1, nog niet uitgevoerd)

Voorwaarde: B1–B4 besloten en root-integratie door release.

```text
qemu-system-x86_64 -machine pc-q35-8.2 -cpu Nehalem-v1 -accel tcg -smp 1 -m 8G \
  <eigen OVMF CODE/VARS-kopie, eigen loader/kernel-image> \
  -audiodev wav,id=snd0,path=<wegwerpmap>/out.wav,out.frequency=48000,out.channels=2,out.format=s16 \
  -device virtio-sound-pci,addr=08.0,audiodev=snd0,streams=1
```

Normaal: de broker speelt een vast, in de gast gegenereerd patroon af; het
gastlog meldt feature-bits, config, gekozen stream, aantal completions en
fence-ACK.

**Orakel (release B8, vóór de gate vastgelegd en achteraf niet te verruimen):**

- Patroon: precies 48 000 frames (1 s), 2 kanalen, S16 little-endian,
  interleaved L,R. Frame `n` (0 ≤ n < 48 000): `L = n as u16`,
  `R = (n as u16) ^ 0x8000`, beide als i16 opgeslagen. Frame 0 is daardoor
  niet stil (`R = 0x8000`).
- De WAV wordt als container geparsed (RIFF/WAVE, `fmt `: PCM, 2 kanalen,
  48 000 Hz, 16 bits; `data`-chunk). Headerbytes worden niet als geheel
  vergeleken; een afwijkend `fmt ` is een fail.
- De `data`-payload moet bestaan uit: een aanloop van 0 tot en met 4 800
  geheel stille frames (≤ 100 ms), daarna de 48 000 patroonframes
  **aaneengesloten en bytegelijk**, daarna 0 tot en met 4 800 stille frames.
  Elke andere byte, gat, herhaling, ontbrekend frame of overschrijding van
  een venster is een fail.
- Aanname die de proef zelf toetst: QEMU 10.2.1 geeft S16/48 kHz/stereo zonder
  resampler of mixerwijziging door. Faalt exactheid, dan blokkeert dat de
  pass; de vensters of de tolerantie worden niet achteraf groter gemaakt.
- Het resultaat telt alleen als N/emulatoruitvoer-deelbewijs, niet als
  hoorbare of fysieke H03/FR-H02/A06-aftekening.

Negatief: F1 (geen device), F13 (broker beëindigd midden in afspelen →
fence-ACK). Controleer transportquiescentie via de beide read-backs en een
geordende used-ring-snapshot met vervolgcontrole op verandering, onafhankelijk
van de WAV-uitvoer. Na de ACK mag nog reeds
vóór revoke geaccepteerde PCM uit de hostqueue volgen, in oorspronkelijke
volgorde en zonder replay; leg vast welke frames vóór revoke zijn geaccepteerd
en waar de ACK in de uitvoer ligt. Die staart en eventuele daaropvolgende
stilte omvatten samen maximaal 4 800 frames na de ACK. Kan de proef de
ACK-positie of herkomst niet aantonen, dan is F13 onbeslist, geen pass.
F14 (slechte buffer), F10 (dubbele RELEASE wordt vóór verzenden geweigerd).
Vastleggen: broncommit, `tools/common.py`-fingerprint,
kernel-/loader-/imagehashes, QEMU/OVMF-hashes, exact commando, wall-clock en
aantal completions. H03 en FR-H02/A06 blijven open.

## 8. Open besluiten (gebundeld voor 14/2/4)

Antwoord release (2ab17599), 30 okt 2026, eenmalig gelezen op `5223214`;
**geen root-integratievrijgave.** Security (081d6f31), 30 okt 2026, eenmalig
gelezen op `5223214`: contractconsistentie, **geen** fysiek fencebewijs,
kwetsbaarheidsaudit of implementatiegoedkeuring. Antwoord van 14: nog open.

| # | Besluit | Eigenaar | Stand |
| --- | --- | --- | --- |
| B1 | Nieuw manifest-only, niet-delegeerbaar uitvoerrecht in capability-core (waarde/naam) | release/kernel + security | Principe akkoord als ontwerpvoorstel (release). Naam/bit, issuer, check, revoke en cleanup nog met kernel + security; **geen gealloceerde capability**. Security: ja als ontwerp, zonder IPC-, microfoon- of beheerimplicatie; de echte cap-/API-wijziging blijft een aparte REL-/kernelclaim. |
| B2 | Syscallnummers en ABI-minor (0x73 is bezet door native-clock) | release | **Niet vrijgegeven.** 0x73 bezet; ABI-allocatie pas op bevestigde integratiebron. |
| B3 | Transport in de kernel (zoals M5/net) bevestigen; geen user-DMA | kernel + security | Security: ja, als gelabeld **vertrouwd** QEMU-kerneltransport met kernelcopy en gepind RAM; **geen isolatie**. Kernel: open. |
| B4 | Featureprofiel en BDF `00:08.0` vastleggen in VM-FR-manifest | release | `00:08.0` alleen voorlopig voor VM-SND-1; niet gereserveerd in de VM-FR-deviceinventaris. |
| B5 | rxq configureren zonder buffers of uitschakelen; QEMU-gedrag per runtimeproef | TL-HW | Open (TL-HW, runtimeproef). |
| B6 | VM-FR `streams=1` (geen invoerstream) in plaats van QEMU-default 2 | release + security | Akkoord voor begrensde output-only VM-SND-1 (release): geen RX-buffers, geen microfoonrecht. Finale VM-FR pas bij integratie. Security: ja voor dit vaste QEMU-profiel, mits native config-/directioncontrole weigert bij een niet-output-only profiel; geen algemene claim dat stream 1 altijd uitvoer is. |
| B7 | Periode/slotgrootte en latencybudget (pas meetbaar op hardware) | TL-HW + Materiaal/apps | Open. |
| B8 | WAV-orakel exact of met tolerantie, en of dit als VM-bewijs telt | release/verifier | Besloten (release): exacte PCM, geen tolerantie; vensters in §7. |
| B9 | IOMMU blijft afwezig: documenteren dat geen DMA-containment wordt geclaimd | security | Security: ja, geen IOMMU- of DMA-containment geclaimd. |

### Securitycorrectie op §5 (bindend vóór transportimplementatie)

- Fence = conjunctie voor een **conform** device: status 0 teruggelezen én PCI
  BME-off teruggelezen, op dezelfde gehele devicefunctie, met deadline en
  ordening. De epoch zelf is geen fence.
- In `Fencing` wordt **vóór** reset en cleanup alle queuepublicatie, control-,
  timer- en IRQ-werk bevroren. Slotownership blijft onzeker tot beide ACKs;
  slots worden niet selectief vrijgegeven. Overflow of een fout blijft
  quarantaine.
- Na beide ACKs volgt een snapshot van de used ring via een expliciet
  geordende DMA-read. Elk completionverschil na de ACK leidt tot quarantaine;
  geen selectieve slotrelease en geen epoch wegstempelen.
- Init en herinit nemen geen oude completion of lease over, ook niet bij
  hergebruik van een PID. Capobject-generatie en device-epoch blijven
  gecontroleerd.
- VM-SND-1: het orakel "geen verdere WAV-groei" is niet gelijk aan een
  geannuleerde hostaudioqueue, want die kan al zijn geconsumeerd. Een
  completion betekent niet "hoorbaar" (§4).

Hostmodel van deze regels (alleen modelbewijs): `src/transport_model.rs`,
PR #91.

## 9. Bewijsgrens

Alleen ontwerp op basis van primaire bronnen en bestaande projectcontracten.
Niets is uitgevoerd in een gast. Geen claim van audio-uitvoer, latency,
DMA-isolatie, fysieke hardware of menselijke audit. Terugdraaien: dit bestand
verwijderen; niets hangt ervan af.
