# Afhankelijkheden

Deze graph ordent implementatie op fundering en bewijs, niet op visuele
zichtbaarheid. `→` betekent “vereist vóór volwaardige acceptatie van”.

```text
Bootbare release
├── UEFI/loader + imageintegriteit
├── kernelstart + memory map
├── paging/W^X + exception/fault policy
├── proces-/service lifecycle + scheduler
├── capability-/IPC-/usercopycontract
└── reproducible build + artifact hashes

Beveiligd gebruikersprofiel
├── entropy + keyring
├── auth/session broker + account lifecycle
├── lokale profielopslag + cold recovery
├── lock/revoke + rate limiting + recovery
└── toegankelijke onboarding en foutflow

Veilig persoonlijk materiaal
├── blokbroker + flush/Unknown-semantiek
├── volumeformaat/admission + journaling/recovery
├── authenticated encryption + sleutelisolatie
├── object-ID/catalogus/revisies + conflictbeleid
├── onafhankelijke backup/export + restore
└── native Materiaal/editor + zoeken + apprechten

Dagelijkse desktop
├── framebuffer/compositor + fonts/rendering
├── input/focus + window/surface lifecycle
├── app SDK + package verify/install/launch/update/remove
├── accessibility tree/events + toetsenbord-only + screenreader
├── settings/session + shutdown/resume
└── first-party apps + bruikbare bestandsformaten

Veilig internet en browser
├── NIC-driver + begrensde DMA/reset
├── native packet/IP/TCP/UDP + DHCP/routes + DNS
├── entropy/keyring + onderhouden TLS/certificaatketen
├── peer-/bestemmingsgebonden capabilities + revoke
├── browserengine + download/open-with portalen
└── traffic capture + negatieve tests + updatebare engine

Persoonlijke NAS/multi-device
├── auth/key/device identity + pairing
├── native internet + TLS/peer authority
├── client-side encrypted object store
├── sync journal + offline reconciliation/conflicts
├── admin quota/revoke/delete onafhankelijk van gast
└── onafhankelijke lokale backup + sleutelherstel

Lokale AI
├── gecontroleerde modelherkomst/licentie/format
├── bounded artifact store + isolated downloader
├── model runtime op benoemde CPU/GPU/NPU
├── expliciete context/task capability
├── resource limits + cancel + accessibility
└── native netwerkdeny voor inference en helpers

Consumer release
├── alle kritieke bovenstaande clusters geïntegreerd
├── installer + veilige herstelroute
├── hardware-/energieprofiel en supportmatrix
├── menselijke accessibility/usability/security beoordeling
├── update-/support-/incidentproces + SBOM/licenties
└── finale hashgebonden image en reproduceerbaar testpakket
```

### Kritieke paden

1. **Data safety:** `storage broker → format/admission → encrypted volume →
   cold recovery → backup/restore → Material save/reopen`.
2. **Authenticatie:** `entropy/keyring → auth broker → account/session →
   lock/unlock → service-specific grants → revoke`.
3. **Internet:** `driver → network config/DNS → TLS identity → per-app egress
   → browser/download → update`.
4. **NAS:** `device identity → pairing → encrypted object layer → sync/conflict
   → admin revoke/delete → independent recovery`.
5. **Daily usability:** `installer → first boot → accessible session →
   settings/input/display → apps/formats → updates/recovery`.

Een mock of testbackend kan interface-/productonderzoek ondersteunen, maar
vervangt geen knoop in deze native dependency graph.
