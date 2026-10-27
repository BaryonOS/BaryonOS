# M5-transportreview en gerichte herstelcontrole

Status na delta-herlezing: alle drie bevindingen op bron gesloten; geen resterende concrete blocker in deze afgebakende delta. Init-readback/device1/exit-na-error zijn hiermee geen uitgevoerde gastproeven. Alleen dit verslag is geschreven. Geen kernel-/driverwijzigingen, builds of QEMU-runs door reviewer.

Basis HEAD bij verslag: `2bb856bfba0f49164b17459c6ff56aa2862007d9`. Werkboom bevat gelijktijdige builderwijzigingen; onderstaande SHA256 identificeert de gelezen relevante bron, geen volledige releasefingerprint.

| Bestand | SHA256 |
|---|---|
| kernel/src/m2/m3/m5.rs | C326EACAA2E1FF84D4CBFC2DA7785E397A594FF99A12C8EFE022E5CD2223F46C |
| drivers/virtio-blk/src/lib.rs | 9E2B8B72ADB1A0C4C98D60BB1BAFBC16D8F9D6205F2B27D2683F9DF5628B9CBA |
| kernel/src/m2/paging.rs | 62E38B8E3E771F5CC375679FB924137869E439554BACD8C1C7DB0938AAA2ED56 |

## Scope

Gelezen: kernel/AGENTS.md, drivers/AGENTS.md, M5-BLOCK-SERVICE-contract, actuele M5-integratie, relevante M3-dispatch/cleanup, MMIO-mapping en M4-copyhelpers, virtio-blk-kerneltransport. Eigen capability-core/virtio-core niet opnieuw als onafhankelijk reviewer onderzocht. Focus: DMA-eigenaarschap en quiescence, completionbounds, PCI/BAR/MMIO, getypeerde autoriteit/ticketbinding en outputcommit.

## Oorspronkelijke bevindingen, hieronder op bron gesloten

1. **Initialisatiefout na BME-enable laat DMA-autorisatie aan.** `drivers/virtio-blk/src/lib.rs:239–242`: na PCI BME-enable kan DRIVER_OK-readback afwijken en initialize direct Err geven. `m5.rs:166–168` of `515–519` classificeert offline/quarantaine zonder BME-clear/reset in deze fouttak. Trigger: weigering DRIVER_OK of NEEDS_RESET tijdens laatste readback. Pool blijft gepind; geen bewezen user-DMA of kwaadaardig-devicecontainment. Vereist herstel: foutpad na BME-enable vraagt reset en wist BME; geen poolreuse tot echte status0-ack. Nieuwe poging mag geen impliciete quiescence aannemen.

2. **Device-id uit scope wordt niet aan runtime-device gekoppeld.** `m5.rs:322–344`: autorisatie controleert type/READ en sectorbereik, maar niet `auth.scope().device`; iedere geldige BlockDevice-cap bereikt runtime-device0. Tegenvoorbeeld: vertrouwde manifestcode maakt scope.device=1, waarna die cap device0 leest. Huidige manifest maakt alleen device0 en ring3 kan geen object creëren: huidig bereik is een integratie-invariant, geen aangetoonde ring3-escalatie. Vereist herstel: expliciete device-id-match vóór transportgebruik, inclusief pollbinding.

3. **Verweesde terminale fout blokkeert volgende submitters na exit.** `m5.rs:424–435`: detach kijkt alleen naar policy.request. Na completionfout/timeout staat de Request ook in r.error; reset_completed(true) op511 verwijdert de policy-request. Exit van die PID vóór de terminale poll laat r.error staan, waardoor submit op341–342 EAGAIN blijft geven aan iedere geldige READ-houder. Trigger: read → fout → resetack → owner-exit vóór poll. Vereist herstel: detach wist ook r.error met overeenkomende binding.pid; bij reeds quiescent/Ready is daarvoor geen nieuwe DMA-reset nodig. Herstart van niet-pending dienstexits is een afzonderlijke beperkte M5-policy.

## Gelezen positieve invarianten

- Vijf afzonderlijke Owner::Kernel-pages blijven bootlang gepind; descriptors, request/status en bounce worden nooit userframes. Detach vraagt reset vóór usercleanup en geeft de pool niet vrij.
- Normale resetprogress herinitialiseert/zeroët uitsluitend na status0 plus BME-off-observatie. Resettimeout houdt pool offline/in quarantaine. BME-off wordt niet zelfstandig als quiescence gebruikt.
- Used-ringadres komt uit eigen old_idx modulo8; onbetrouwbaar used-id wordt niet als pointer/index gebruikt. Delta/head/len/status worden vóór bouncecopy gecontroleerd. Alleen volledige succesvolle512B+status levert data; IOERR/UNSUPP/protocolfouten leveren geen payload.
- Poll herautoriseert actuele PID/type/READ/objectkey en ticket; volledige output320 wordt vooraf op effectieve USER/RW en Owner::User gecontroleerd. Bytewise scratchcopy vermijdt misaligned-u64-dereference. Ongeldige output consumeert geen completion.
- MMIO gebruikt afzonderlijke supervisor RW/NX-mappings, PCD/PWT en gecontroleerde PAT-index3=UC. Geen usergekozen PA/MMIO/descriptorroute. Normale RAM-scratchmapping behoudt executable-frameverbod.

## Bewijsgrenzen

Dit is bronreview door een tweede AI, geen onafhankelijke menselijke audit. QEMU-uitvoering en samengevoegde regressiepoort staan bij root/builder; deze reviewer heeft die niet uitgevoerd. Lokale completion-snapshotinjecties na een echte completion bewijzen foutafhandeling, geen kwaadaardige DMA-containment. Used bevat geen epoch/nonce; softwareticketepochs bewijzen geen hardware-replaydetectie. Protocolconforme QEMU-device/backend en kleine kerneltransportdriver blijven TCB. Geen VT-d-, echte hardware-, schrijf-, netwerk- of cryptografieclaim.

## Gerichte delta-herlezing na herstelmelding

Gelezen op HEAD `42608d799643cb0cfd4308ca1069deeb7490d298`; bron was door builder bevroren en root maakte het checkpoint. Alleen de drie herstelpaden, quarantine-cleanup en CPUID-breedtevalidatie met relevante context herlezen. Geen nieuwe brede scan.

| Bevinding/delta | Actuele bron en resultaat |
|---|---|
| Initfout na BME-enable | driver:241–247 vraagt status0-reset en wist BME vóór Err. m5.rs:194–196 en689–692 zetten offline/quarantine. Pool blijft gepind en wordt niet opnieuw gebruikt. **Op bron gesloten.** |
| Verkeerd scope-device | m5.rs:364–366 weigert scope.device!=0 met EACCESS vóór de gezamenlijke submit/pollroute en vóór transport. **Op bron gesloten.** |
| Terminale fout na PID-exit | m5.rs:470–474 wist r.error voor de stervende PID onafhankelijk van policy.request; de oorspronkelijke pending-resetroute blijft apart. **Op bron gesloten.** |
| Cleanup in quarantine | m5.rs:475–477 keert terug vóór start_reset. De metadatafout kan worden gewist, maar gepinde pool/transport blijven onaangeraakt. Geen resetaanroep op onherstelbare Quarantined-policy. **Bronpad beoordeeld.** |
| CPU-fysieke adresbreedte | paging.rs:41–51 leest aanwezige CPUID MAXPHYADDR en weigert ontbrekende/ongeldige breedte. kernel-core:7–12 controleert32..52, pagealignment, checked page-end en de CPUgrens; tests236–243 dekken32/36-bitgrens, overflow, onbekende breedte en misalignment. **Bron en testinhoud beoordeeld; niet door reviewer uitgevoerd.** |

Actuele SHA256:

| Bestand | SHA256 |
|---|---|
| kernel/src/m2/m3/m5.rs | B314866324E56930B68733F861F1E1F49E315F5BE33203EA695D374DC8B1CE64 |
| drivers/virtio-blk/src/lib.rs | FAF1ACF051CA0C2E7E09A07B31142C5D01A711C372B0175AB5EC63ADAC938383 |
| kernel/src/m2/paging.rs | 96727D1543B50C13B6925A249AC281ED4C4D8FDC66039E6B70FD97D6BDE8888C |
| crates/kernel-core/src/lib.rs | 3D0303A83B18171C9C2D5C9F1C6448E2BD944AA7F68C5A39FF50F987162B5495 |

Builder meldde case12-hertest `303406d019` raw33, drie-restartcase `f53e180603` raw33 en kernel-core15/15 hosttests. Deze mededelingen zijn niet zelfstandig opnieuw uitgevoerd of als logs door reviewer gecontroleerd. De gecombineerde gate is werk van root en blijft apart bewijs. Er is geen gastinjectie geclaimd van de specifieke initialisatie-readbackfout, device1-scope of exit-na-terminale-fout. Sluiting van die drie bevindingen berust op de gelezen bronfix; dat is geen aanvulling van niet-uitgevoerde gasttests.
