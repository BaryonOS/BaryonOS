# Technische gaten

1. **Geïntegreerde kern:** bevries een releaseprofiel; herhaal boot, process,
   resource, IPC, entropy, storage, network en window gates op exact dezelfde
   bron/image.
2. **Service-/procesmodel:** complete fault boundaries, wait/lifecycle,
   exception policy, SMP/synchronization en bounded logging/timers. De huidige
   M2–M4-contracten/proeven zijn niet de hele productdienstarchitectuur.
3. **Drivers:** van testvirtio/PS2 naar gedocumenteerde NIC, display, audio,
   USB, storage, battery/thermal en suspend/resume. Beschrijf DMA/IOMMU/
   resetbeperkingen zonder ze weg te redeneren.
4. **Storage:** bestaand objectvolume is geen algemeen gebruikers-filesystem.
   Volledige format admission/migration, encrypted persistent data, key
   lifecycle, crash/power-loss semantics, backup en independent restore zijn
   nodig.
5. **Network:** native config/DHCP/DNS/routes/TLS/browser/peer rights in één
   gecontroleerde stack; host-transporttest mag ontbrekende OS-client niet
   vervangen.
6. **Application runtime:** package verifier naar installer/launcher,
   dependencies, update/remove, defaults/intents, permissions, background
   jobs, quotas en recovery.
7. **UI/platform:** native accessibility semantics, shaping/font fallback,
   multi-window/display states, settings persistence, usable formats, audio/
   media pipelines.
8. **Release:** SBOM/licence/provenance, reproducible clean build, update
   response, support matrix, clean-environment artifact review en evidence
   traceability.

### Onderhoudslast

Browserengine, codecs, brede filesystem compatibility, Windows/game runtime,
Wi-Fi/Bluetooth chipsets en QPU hardware zijn afzonderlijke zeer grote
onderhoudsprogramma's. Vergelijk onderhouden libraries/engines vóór eigen
implementatie; beheer hun patch cadence, license, no_std/ABI, dependencies,
build scripts en sandbox.
