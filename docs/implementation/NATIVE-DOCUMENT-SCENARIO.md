# TASK-DOC-NATIVE — echte documentsessie in het gezamenlijke systeem

Taakcontract, 29 oktober 2026. Eigenaar appintegratie: `native_ui_apps`;
hoofdagent bezit gedeelde lockfiles, kernelobserver en eindintegratie.
Schrijfgebied: `sdk/examples/system/src/app.rs`, het pakketmanifest en dit
document. Geen kernel-/dienst-/SDK-protocolwijziging of grafische productclaim.

## Native scenario en eigenaarschap

De gezamenlijke systeemapp gebruikt voor werkelijk documentwerk
`sdk/document-session::volume::{DocumentSession, VolumeSession, Buffers}` en
de bestaande `NativeTransport`. De v7-uploadtag en een dienstuitgegeven token
verbinden BEGIN/APPEND/ABORT/COMMIT aan de via de kernel bekende afzender en
het toegekende domein. Het document blijft object 0, `systeemproef`, 4096
fixturebytes, revisie 1. De bestaande recordcryptoproeven blijven ongewijzigd;
deze gegevens op de testdisk blijven plaintext en bewijzen geen B02-encryptie.

De oude v6-client doet uitsluitend vóór READY de bestaande STAT- en echte
cross-domain-weigeringsproeven. Zijn lexicale levensduur eindigt vóór de nieuwe
documentsessie begint. Daarna bezit alleen de documentsessie het endpointpaar;
afsluiten sluit beide endpoints expliciet. Geen twee actieve sessies delen
dezelfde handles. De drie callerbuffers van ieder 4096 bytes staan in een
aparte `inline(never)`-functie op de bestaande 64-KiB-appstack, niet in BSS.
Een tijdelijke fixturechunk is maximaal 160 bytes. Er zijn geen sleutels,
blokrechten of hostbestandsroutes toegevoegd.

### Normale start

1. CREATE controleert afwezigheid en maakt uitsluitend een lokaal concept.
2. De publieke replace-range-API bouwt de 4096 fixturebytes in het concept.
3. BEGIN + één APPEND van 160 bytes oefenen echte tokengebonden voortgang uit.
   ABORT annuleert vóór de 512-byte-sectorbuffer een contentschrijfopdracht
   kan uitsturen. Het concept blijft vuil, revisie 0 en niet onzeker.
4. SAVE gebruikt een nieuw uploadtoken en request-ID 3; COMMIT moet revisie 1
   bevestigen. Een onzekere of verkeerde reply levert geen APP_DOCS-succes.
5. OPEN leest het werkelijk bevestigde document opnieuw; naam, revisie,
   volledige inhoud en checksum worden gecontroleerd. Daarna volgt close.

### Koude recovery

Een volledig nieuwe VM gebruikt exact de normale disk, met readonly-QEMU-drive
en het bestaande lees-only blokrecht. De SDK opent het document in ReadOnly,
controleert opnieuw alle 4096 bytes en weigert lokale edit/save zonder
mutatie. De onafhankelijke schijforakel en identieke before/afterhashes blijven
de runnervoorwaarden; geen snapshotherstel of herstelde RAM-toestand.

## Acceptatie en bewijsgrens

Bouw het gewone gebruikersprogramma voor `x86_64-unknown-none` met het bestaande
bouwscript. Draai daarna `tools/test_system.py` op één bevroren bron: twee echte
QEMU-runs (normal/recover), echte IPC, alle bestaande raw-capabilityweigeringen,
recordcontext-/intrekkingsproeven, lifecycle/cleanup en onafhankelijk diskorakel.
De normale eindbuild moet bytegelijk zijn aan de werkelijk geteste normale
kernel. Gedeelde `sdk/Cargo.lock`-aanpassing bevat alleen de bestaande lokale
dependencyverbindingen en wordt door de hoofdagent geïntegreerd.

Deze taak bewijst na geslaagde uitvoering de native SDK-v7-positieve keten en
de genoemde foutweigeringen. Negatieve uploads met token-/afzenderverwisseling
blijven afzonderlijke host-/native gates volgens hun werkelijke bewijsstatus.
Geen Materiaal-GUI, login-/sessielifecycle, persistente sleutels, versleuteld
documentvolume of complete testreleaseacceptatie wordt hiermee geclaimd.

## Werkelijk uitgevoerd — 29 oktober 2026

Basis `d73d515` plus deze appwijziging en de lokaal door hoofdagent toegestane
lockfilegeneratie. Bevroren bronvingerafdruk v3:
`3f5869e80636e2127b37a268fafbcb2b2936626522912fef7aca507bf90457ae`.
De lockfilediff bevat alleen de twee bestaande lokale dependencies van de
systeemapp; de hoofdagent integreert die gedeelde lockfile afzonderlijk.

Met Rust 1.90.0, QEMU 10.0.13, `pc-q35-8.2`, TCG, qemu64, één CPU en 256 MiB:

```sh
PATH=/workspace/.nieuw-os-env/bin:$PATH \
LOCALAPPDATA=/workspace/nieuw-os/.tools \
OS_OVMF_CODE=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_CODE_4M.fd \
OS_OVMF_VARS=/workspace/.nieuw-os-env/sysroot/usr/share/OVMF/OVMF_VARS_4M.fd \
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B -u tools/test_system.py
```

**Exit 0:** negen systeemharnastests, 17 SDK-runtimetests en twee echte VM-runs.
Normal en koude recover geven raw exit 33; beide hebben APP_DOCS=1, revisie 1,
4096 bytes, drie echte capabilityweigeringen, record-seal/unseal 1/1,
contextweigeringen 10, intrekkingsweigeringen 2 en lege uiteindelijke
proces-/capabilitytoestand. Normal: 42 diskwrites/3 flushes. Recover:
0 writes/0 flushes. Het onafhankelijke diskorakel slaagt voor beide images.

Normal-after, recover-before en recover-after zijn alle:
`ad40c83d6b9d0a409391113f7ed4deb5aafd7c27f6629c22d7a6ddc060c639ea`.
De finale normale kernel is bytegelijk aan de geteste normale kernel:
`a6a559bf5a7f02b02cdba55ff4a6676805838981cb94df640acee5c77ed0b8f7`.
De gewone system-app-ELF heeft SHA-256
`a414dfba36e620e8d225c351d023cd5becdff0b130a76e29135c15f745119b4f`;
het geïnspecteerde RW-segment is 1304 bytes en blijft binnen één datapagina.

Targetbuild via `sdk/examples/system/build.py`, gerichte target-Clippy met
`-D warnings`, pakketformatteringscontrole en diffcontrole slagen. Clippy
gebruikte dezelfde portable crypto-cfg's; er zijn geen nieuwe crates opgehaald.
Een afzonderlijke AI-bronreview vond geen concrete blocker binnen ownership,
annulering en foutafhandeling; dit is geen menselijke audit of formele
worstcase-stackgarantie.

Volledig lokaal bewijs: `build/system-gate.json`,
`build/run/system-normal-e9e9d709347b46fbb9585bb111b64015/` en
`build/run/system-recover-a1589c5ca67f440f83b81c185a86d194/`, met rawserial,
manifesten, echte images en orakels. Commandologs staan onder `build/logs/`.
De eerste poging stopte vóór VM-start wegens ontbrekend QEMU-PATH; die
setupfout is bewaard als `build/logs/native-document-no-qemu-gate.json` en
telt niet als native run. Na selectie van de bestaande runtime slaagde de
volledige gate. De hoofdagent herhaalt de samengevoegde bron vóór eindacceptatie.
