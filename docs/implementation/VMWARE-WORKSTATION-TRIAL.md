# VMware Workstation Pro — concrete proefopdracht

2 november 2026. Ik heb VMware Workstation Pro op mijn Windows-pc
geïnstalleerd. De opdracht is een VMware-route onderzoeken naast de bestaande
ontwikkelroute.
Er is nog **geen VMware-uitvoering, ISO of werkende VMware-desktop bewezen**.

De aangewezen uitvoerende sessie is **Ontwerp veilige OS-desktop**,
`01a19c0e-6a17-70e4-8da0-263949812063`, host `durable`. De coördinerende sessie
heeft deze sessie gelezen. De uitvoering was gepauzeerd na mijn verzoek om
afstemming. Deze opdracht is **voorbereid en nog niet verzonden**: in de
coördinerende sessie was alleen `read_thread` beschikbaar, zonder
`send_message_to_thread` of `handoff_thread`. Opslag van deze opdracht betekent
daarom niet dat de uitvoerende sessie is gestart.

## Opdracht voor de uitvoerende sessie

1. Behoud je bestaande WIP en de nieuwe sessieslice in concept-PR129. Neem geen
   overlappende desktop-, auth-, transport- of scorebestanden opnieuw over.
   De geteste sessiebron is `703cc0705471062ffb1483a16f42a2e580a6fdf7`;
   latere documentatie verandert dat brongebonden bewijs niet.
2. Controleer eerst of je werkelijk toegang hebt tot de Windows-pc. Een
   cloudwerkmap `/workspace` is geen toegang tot de geïnstalleerde Workstation.
   Meet lokaal de Workstation-versie, beschikbare `vmrun`, hostvirtualisatie-
   modus en gebruikte backend. Verander geen Windows-, Hyper-V-, VBS-, TPM-
   of firmwarebeveiliging om een snelheidsclaim te halen.
3. Onderzoek de gastcompatibiliteit voordat je een VM start. De huidige
   `native-session-route` vereist virtio-blk `1af4:1042` op `00:05.0` en
   virtio-rng `1af4:1044` op `00:06.0`. Workstation documenteert IDE/SATA/SCSI/
   NVMe; die zijn geen vervanging voor deze drivers. Een disk naar VMDK
   converteren maakt de native Materiaal-desktop daarom niet werkend.
4. Maak eerst een afzonderlijke, begrensde platformproef met het bestaande
   `graphics-g0`-profiel: eigen UEFI-loader, eigen kernel, GOP en gepolde
   PS/2-invoer, zonder gastdataschijf of sleuteldienst. Hergebruik `build.py`,
   het buildmanifest, de bronfingerprint en de gecontroleerde ESP-bestanden.
5. Bouw een reproduceerbare UEFI El-Torito-ISO met een FAT-bootimage dat
   **zowel** `EFI/BOOT/BOOTX64.EFI` als `kernel.elf` bevat. De loader opent de
   kernel vanaf hetzelfde firmwarevolume. De huidige runners leveren alleen
   een ESP-map via QEMU `fat:esp`; er bestaat nog geen generieke ISO-exporter.
   Kies en controleer een onderhouden hosttool met herkomst/versie/licentie.
   Geen fysieke schijf formatteren of mounten als gastopslag.
6. Maak een nieuwe wegwerp-VM, met eigen NVRAM en uitsluitend de gemaakte ISO:
   UEFI, één vCPU, aanvankelijk het geteste 256-MiB-budget, COM1 naar een lokaal
   bestand; geen netwerk, gastdataschijf, gedeelde map, clipboardintegratie,
   USB- of GPU-passthrough en geen 3D. Verifieer welke configuratie de werkelijk
   geïnstalleerde Workstation-versie ondersteunt. Wijzig geen bestaande VM.
7. Controleer de Secure-Boot-toestand en stop met een concrete diagnose als
   de ontwikkelloader wordt geweigerd; zet dit niet stilzwijgend uit. De
   G0-loader vraagt exact 800×600 RGB/BGR-GOP. Een ontbrekende modus is een
   compatibiliteitsfout, geen aanleiding om succes te simuleren.
8. Bewaar COM1-log, werkelijk beeld en toetsreactie, VMX/ISO-/bronhashes,
   versie/backend, tijd tot `G0:READY`, screenshot en normale stop. Gebruik
   eerst Workstation **VM → Capture Screen**. `vmrun captureScreen` vereist
   volgens de onderzochte handleiding een geldige gastlogin; Nieuw OS heeft
   geen VMware Tools-/loginadapter. Geen werkende automatisering claimen.
9. Vergelijk pas daarna dezelfde boot-/renderworkload tussen VMware en QEMU.
   Noem de daadwerkelijk gebruikte QEMU-accelerator: TCG en Windows/WHPX zijn
   verschillende profielen. Workstation is niet zonder meting sneller.

Acceptatie van deze eerste proef betekent alleen UEFI/GOP/PS2 op VMware.
De geïsoleerde sessiedesktop vereist daarna geschikte opslag-/entropy-adapters,
begrensde apparaat-/DMA-contracten en eigen negatieve proeven. Behoud QEMU als
bestaande regressie- en acceptatieroute totdat VMware zelfstandig bewijs heeft.

## Uitvoeromgeving en grenzen

De actieve host is Linux x86_64. VMware, `vmrun`, `xorriso`, `mformat`, `mcopy`
en `mkfs.fat` zijn hier niet aangetroffen, ook niet na activeren van de
projectomgeving. Er is geen uitvoertoegang tot de Windows-pc aangetoond.
Er is geen Windows-instelling, echte disk of bestaande VM gewijzigd.

Primaire bronnen, geraadpleegd op 2 november 2026:

- Broadcom, [Using VMware Workstation Pro 17](https://techdocs2-prod.adobecqms.net/content/dam/broadcom/techdocs/us/en/pdf/vmware/desktop-hypervisors/workstation/vmware-workstation-pro-17-0.pdf):
  controllers, firmware, PS/2, Host VBS Mode en screenshotvoorwaarden
  (`captureScreen`, p.314). Dit document bewijst niet welke versie op mijn pc
  is geïnstalleerd.
- Broadcom, [Workstation met Hyper-V/VBS en WHP](https://knowledge.broadcom.com/external/article?legacyId=76918):
  ondersteuning hangt van de hostconfiguratie af; lokaal vaststellen.
- UEFI Forum, [UEFI 2.11, Media Access](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html):
  FAT en El-Torito-boot.

Broncode voor de huidige apparaatgrenzen:
`drivers/virtio-blk/src/lib.rs::discover`,
`drivers/virtio-rng/src/lib.rs::discover`,
`kernel/src/graphics.rs` en `boot/loader/src/main.rs`.
