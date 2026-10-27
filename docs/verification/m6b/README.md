# M6b — definitieve native opslagproef

Broncommit `1e61a20beac67d685cf4bff5f0f9f588bf2cb56e`, fingerprint v3
`43e6c06e43e3ba602b49c304328f0c1700bc26dae0a1256f0d5d420421f55ae2`.
De volledige gate en capture zijn op 28 oktober 2026 (UTC-logdatum 27 oktober)
met exit 0 afgerond:
**18 controlegroepen, 14 echte VM-invocaties, 22 Rust-tests en 26 harnastests**.
Oudere geslaagde checkpointuitvoeringen worden niet bij deze aantallen opgeteld.

## Werkelijke uitvoering

- Twee normale mutatieproeven op verschillende synthetische volumes met dezelfde
  binaries; elk krijgt een afzonderlijke koude herstel-VM.
- CRUD- en domeinfouten, gevolgd door koude recovery zonder schijfwijzigingen.
- 32 echte journalwrites, daarna weigering bij uitputting; koude recovery.
- Een beschadigd journal wordt vóór READY afgewezen.
- Twee beëindigingen van nog levende QEMU-processen: vóór de eerste WRITE en na
  de eerste daadwerkelijke mutatie-FLUSH maar vóór ACK. Beide krijgen een nieuwe
  herstel-VM op exact dezelfde image.
- Een afsluitende gewone M1-boot, gebonden aan zijn verwachte kernel/loaderhashes.

Dat zijn zeven volumes, zes koude herstelparen, twee externe kills en één M1-run
binnen de veertien invocaties. De eigen ring-3-service voert het objectbeleid uit;
de kernelobserver controleert echte IPC-, device- en client-ACK-overgangen.

## Bewijsbestanden

`suite.json` bewaart de volledige groepsvolgorde. `guest-runs.json` bevat alle
veertien afzonderlijke records met ruwe uitvoer, exitstatus, bron, commando,
artifacthashes en schijforakel. `capture.json` bewaart de opnieuw gecontroleerde
archiefhashes. De drie hostlogtrio's bevatten exact uitgevoerde commando's,
werkdirectories, exitstatussen en uitvoer.

`gate.json` en `gate.stdout.txt` bewaren de gateopdracht en console. De eerste
`capture-command.stdout.txt` bleek leeg; daaruit wordt geen succesmarker afgeleid.
De extra capture-hercontrole in `../m6b-review/capture-recheck/` heeft volledige
vastgelegde stdout en exit 0 voor M1 en M6b, zonder nieuwe VM-runs.

De afzonderlijke audit in `../m6b-review/final-native-audit.json` herberekende
121 artifacthashes, reconstrueerde de bronfingerprint uit Git en decodeerde alle
werkelijke journal-/payload-/canarybytes zonder de projectparser te importeren.
Dit is een afzonderlijke AI-controle, geen menselijke audit of formeel bewijs.

## Grenzen en reproductie

Vier objecten, twee domeinen, maximaal 128 bytes per object en 32 append-only
records. Geen namen, directories, ruimtehergebruik, encryptie, native NAS of
desktopintegratie. DELETE is logisch: historische payloadbytes blijven staan.
FNV is diagnostiek, geen cryptografische integriteit. QEMU/device/kerneltransport
zijn vertrouwd; een proceskill bewijst geen fysieke stroomuitval of IOMMU-isolatie.
Unknown-uitkomsten zijn aanvullend met zeven host-ABI-proeven gecontroleerd,
niet met native M6b-devicefoutinjectie.

```sh
python3 -B -u tools/test_m6b.py
python3 -B tools/capture_m6b.py
```

Toolchain en vaste QEMU/OVMF-configuratie: `../native-linux-toolchain/`.
Contract: `../../implementation/M6B-OBJECT-STORAGE.md`.
